use gstd::ActorId;
use gtest::{Log, Program, System};
use io::{Action, MessageAction, Event};

const USER: u64 = 3;
const TARGET_PROGRAM_ADDRESS: u64 = 2;

#[test]
fn test() {
    let system = System::new();
    system.init_logger();

    let proxy_program = Program::current(&system);
    let target_program = Program::from_file(&system, "target/wasm32-unknown-unknown/debug/target_program.opt.wasm");

    let result = target_program.send_bytes(USER, []);
    assert!(!result.main_failed());
    let target_program_address: ActorId = TARGET_PROGRAM_ADDRESS.into();
    let result = proxy_program.send(USER, target_program_address);
    assert!(!result.main_failed());

    let result = proxy_program.send(USER, Action::SendMessage(MessageAction::MakeRandomNumber{range: 1}));
    assert!(!result.main_failed());

    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(Event::MessageSent);

    assert!(result.contains(&log));

    let result = proxy_program.send(USER, Action::SendMessage(MessageAction::MakeRandomNumber{range: 1}));
    assert!(!result.main_failed());

    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(Event::WrongStatus);

    assert!(result.contains(&log));

    system.spend_blocks(3);

    let mailbox = system.get_mailbox(USER);
    let log = Log::builder()
        .source(1)
        .dest(3)
        .payload(Event::NoReplyReceived);

    assert!(mailbox.contains(&log));
}