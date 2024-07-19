#![no_std]

use gstd::{debug, msg, exec, ActorId, MessageId};
use io::{Action, Event};

static mut SESSION: Option<Session> = None;

type SentMessageId = MessageId;
type OriginalMessageId = MessageId;

#[derive(PartialEq)]
enum SessionStatus {
    Waiting,
    MessageSent,
    ReplyReceived(Event),
}

#[derive(PartialEq)]
struct Session {
    target_program_id: ActorId,
    msg_ids: (SentMessageId, OriginalMessageId),
    session_status: SessionStatus,
}

#[no_mangle]
extern "C" fn init() {
    let target_program_id = msg::load().expect("Unable to decode Init");
    unsafe {
        SESSION = Some(Session {
            target_program_id,
            msg_ids: (MessageId::zero(), MessageId::zero()),
            session_status: SessionStatus::Waiting,
        });
    }
}

#[no_mangle]
extern "C" fn handle() {
    debug!("!!!! HANDLE !!!!");
    debug!("Message ID: {:?}", msg::id());
    let action: Action = msg::load().expect("Unable to decode `Action`");
    debug!("Message payload: {:?}", action);
    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };

    // match session_status
    match &session.session_status {
        SessionStatus::Waiting => {
            debug!("HANDLE: SessionStatus::Waiting");
            let msg_id = msg::send(session.target_program_id, action, 0)
                .expect("Error in sending a message");
            debug!("HANDLE: SessionStatus::Sent");
            session.session_status = SessionStatus::MessageSent;
            session.msg_ids = (msg_id, msg::id());
            debug!("HANDLE: WAIT");
            exec::wait();
        }
        SessionStatus::MessageSent => {
            debug!("HANDLE: SessionStatus::MessageSent");
            msg::reply(Event::MessageAlreadySent, 0).expect("Error in sending a reply");
        }
        SessionStatus::ReplyReceived(reply_message) => {
            debug!("HANDLE: SessionStatus::ReplyReceived({:?})", reply_message);
            msg::reply(reply_message, 0).expect("Error in sending a reply");
            session.session_status = SessionStatus::Waiting;
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
        let original_message_id = session.msg_ids.1;
        debug!("HANDLE: WAKE");
        exec::wake(original_message_id).expect("Failed to wake message");
    }
}