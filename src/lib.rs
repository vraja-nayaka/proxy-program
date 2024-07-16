#![no_std]

use gstd::{msg, ActorId, MessageId};
use io::{Action, Event};

static mut SESSION: Option<Session> = None;

struct Session {
    target_program_id: ActorId, // target program address
    msg_id_to_actor_id: (MessageId, ActorId), // tuple containing the identifier of a message sent to a Target program and the Id of a User initiating the action
}

#[no_mangle]
extern "C" fn init() {
    let target_program_id: ActorId = msg::load().expect("Unable to decode Init");
    unsafe {
        SESSION = Some(Session {
            target_program_id,
            msg_id_to_actor_id: (MessageId::zero(), ActorId::zero()),
        });
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: Action = msg::load().expect("Unable to decode ");
    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };
    let msg_id = msg::send(session.target_program_id, action, 0).expect("Error in sending a message");
    session.msg_id_to_actor_id = (msg_id, msg::source());
    msg::reply(Event::MessageSent, 0).expect("Error in sending a reply");
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_message_id = msg::reply_to().expect("Failed to query reply_to data");
    let session = unsafe { SESSION.as_mut().expect("The session is not initialized") };
    let (msg_id, actor) = session.msg_id_to_actor_id;
    if reply_message_id == msg_id {
        let reply: Event = msg::load().expect("Unable to decode ");
        msg::send(actor, reply, 0).expect("Error in sending a message");
    }
}