use core::fmt::Debug;

use super::*;

#[derive(Debug, Clone)]
enum WriterTransition {
    Read(usize),
    WriteUpTo(usize),
    Reset,
}

struct WriterSM;

impl ReferenceStateMachine for WriterSM {
    type State = Status;
    type Transition = WriterTransition;

    fn init_state() -> BoxedStrategy<Self::State> {
        strategy::Just(Status::new(CAP)).boxed()
    }

    fn transitions(_state: &Self::State) -> BoxedStrategy<Self::Transition> {
        prop_oneof![
            (1..50_usize).prop_map(WriterTransition::Read),
            (1..50_usize).prop_map(WriterTransition::WriteUpTo),
            strategy::Just(WriterTransition::Reset),
        ]
        .boxed()
    }

    fn apply(status: Self::State, transition: &Self::Transition) -> Self::State {
        match (status, transition) {
            (_, WriterTransition::Reset) => Status::Available(CAP),
            (Status::Available(x), WriterTransition::Read(y)) => {
                if x < *y {
                    Status::Failed
                } else {
                    Status::Available(x - y)
                }
            }
            (Status::Failed, WriterTransition::Read(_)) => Status::Failed,
            (Status::Available(x), WriterTransition::WriteUpTo(y)) => Status::Available((x + *y).min(CAP)),
            (Status::Failed, WriterTransition::WriteUpTo(_)) => Status::Available(CAP),
        }
    }
}

struct WriterSut {
    status: Status,
    buffer: *mut [u8],
    producer: WritableDmaRingBuffer<'static, u8>,
    consumer: DmaMock,
}

impl Debug for WriterSut {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        <DmaMock as Debug>::fmt(&self.consumer, f)
    }
}

struct WriterTest;

impl StateMachineTest for WriterTest {
    type SystemUnderTest = WriterSut;
    type Reference = WriterSM;

    fn init_test(ref_status: &<Self::Reference as ReferenceStateMachine>::State) -> Self::SystemUnderTest {
        let buffer = Box::into_raw(Box::new([0; CAP]));
        WriterSut {
            status: ref_status.clone(),
            buffer,
            producer: WritableDmaRingBuffer::new(unsafe { &mut *buffer }),
            consumer: DmaMock::default(),
        }
    }

    fn teardown(state: Self::SystemUnderTest) {
        unsafe {
            let _ = Box::from_raw(state.buffer);
        };
    }

    fn apply(
        mut sut: Self::SystemUnderTest,
        ref_status: &<Self::Reference as ReferenceStateMachine>::State,
        transition: <Self::Reference as ReferenceStateMachine>::Transition,
    ) -> Self::SystemUnderTest {
        match transition {
            WriterTransition::Read(x) => sut.consumer.advance(x),
            WriterTransition::Reset => {
                sut.producer.reset(&mut sut.consumer);
            }
            WriterTransition::WriteUpTo(x) => {
                let status = sut.status;
                let WriterSut {
                    ref mut producer,
                    ref mut consumer,
                    ..
                } = sut;
                let mut buf = vec![0; x];
                let res = producer.write(consumer, &mut buf);
                match status {
                    Status::Available(n) => {
                        let writable = x.min(CAP - n.min(CAP));
                        assert_eq!(res.unwrap().0, writable);
                    }
                    Status::Failed => assert!(res.is_err()),
                }
            }
        }

        WriterSut {
            status: ref_status.clone(),
            ..sut
        }
    }
}

prop_state_machine! {
    #[test]
    fn writer_state_test(sequential 1..20 => WriterTest);
}
