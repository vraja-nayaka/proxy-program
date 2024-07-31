#![no_std]

use gstd::{debug, msg, exec, ActorId, MessageId};
use io::{Action, Event};

static mut SESSION: Option<Session> = None;

type MessageSentId = MessageId;
type OriginalMessageId = MessageId;

#[derive(PartialEq)]
enum SessionStatus {
    Waiting,
    MessageSent,
    ReplyReceived(Event),
}

#[derive(PartialEq)]
struct Session {
    target_program: ActorId,
    msg_ids: (MessageSentId, OriginalMessageId, ActorId),
    session_status: SessionStatus,
}

#[no_mangle]
extern "C" fn init() {
    let target_program = msg::load().expect("Unable to decode Init");
    unsafe {
        SESSION = Some(Session {
            target_program,
            msg_ids: (MessageId::zero(), MessageId::zero(), ActorId::zero()),
            session_status: SessionStatus::Waiting,
        });
    }
}

#[no_mangle]
extern "C" fn handle() {
    debug!("!!!! HANDLE !!!!");
    debug!("Message ID: {:?}", msg::id());
    let action: Action = msg::load().expect("Unable to decode ");
    debug!("Message payload: {:?}", action);
    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };

    match action {
        Action::SendMessage(message_action) => {
            if session.session_status == SessionStatus::Waiting {
                debug!("HANDLE: Action::SendMessage and SessionStatus::Waiting");
                let msg_id = msg::send(session.target_program, message_action, 0)
                    .expect("Error in sending a message");
    
                debug!("HANDLE: SessionStatus::MessageSent");
                session.session_status = SessionStatus::MessageSent;
                session.msg_ids = (msg_id, msg::id(), msg::source());

                msg::send_delayed(exec::program_id(), Action::CheckReply, 0, 3)
                    .expect("Error in sending a message");

                msg::reply(Event::MessageSent, 0).expect("Error in sending a reply");

            } else {
                debug!("HANDLE: Event::WrongStatus");
                msg::reply(Event::WrongStatus, 0).expect("Error in sending a reply");
            }
        }
        Action::CheckReply => {
            debug!("HANDLE: Action::CheckReply");
            if session.session_status == SessionStatus::MessageSent && msg::source() == exec::program_id() {
                debug!("HANDLE: No response was received");
                msg::send(session.msg_ids.2, Event::NoReplyReceived, 0).expect("Error in sending a message");
                debug!("HANDLE: SessionStatus::Waiting");
                session.session_status = SessionStatus::Waiting;
            }
        }
    }
    debug!("HANDLE: END");
}

#[no_mangle]
extern "C" fn handle_reply() {
    debug!("HANDLE_REPLY");
    let reply_to = msg::reply_to().expect("Failed to query reply_to data");
    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };

    if reply_to == session.msg_ids.0 && session.session_status == SessionStatus::MessageSent {
        let reply_message: Event = msg::load().expect("Unable to decode `Event`");
        debug!("HANDLE_REPLY: SessionStatus::ReplyReceived {:?}", reply_message);
        session.session_status = SessionStatus::ReplyReceived(reply_message);
    }
}
