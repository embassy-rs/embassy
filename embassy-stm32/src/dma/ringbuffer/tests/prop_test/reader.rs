use core::fmt::Debug;

use super::*;

#[derive(Debug, Clone)]
enum ReaderTransition {
    Write(usize),
    Reset,
    ReadUpTo(usize),
}

struct ReaderSM;

impl ReferenceStateMachine for ReaderSM {
    type State = Status;
    type Transition = ReaderTransition;

    fn init_state() -> BoxedStrategy<Self::State> {
        strategy::Just(Status::new(0)).boxed()
    }

    fn transitions(_state: &Self::State) -> BoxedStrategy<Self::Transition> {
        prop_oneof![
            (1..50_usize).prop_map(ReaderTransition::Write),
            (1..50_usize).prop_map(ReaderTransition::ReadUpTo),
            strategy::Just(ReaderTransition::Reset),
        ]
        .boxed()
    }

    fn apply(status: Self::State, transition: &Self::Transition) -> Self::State {
        match (status, transition) {
            (_, ReaderTransition::Reset) => Status::Available(0),
            (Status::Available(x), ReaderTransition::Write(y)) => {
                if x + y > CAP {
                    Status::Failed
                } else {
                    Status::Available(x + y)
                }
            }
            (Status::Failed, ReaderTransition::Write(_)) => Status::Failed,
            (Status::Available(x), ReaderTransition::ReadUpTo(y)) => Status::Available(x.saturating_sub(*y)),
            (Status::Failed, ReaderTransition::ReadUpTo(_)) => Status::Available(0),
        }
    }
}

struct ReaderSut {
    status: Status,
    buffer: *mut [u8],
    producer: DmaMock,
    consumer: ReadableDmaRingBuffer<'static, u8>,
}

impl Debug for ReaderSut {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        <DmaMock as Debug>::fmt(&self.producer, f)
    }
}

struct ReaderTest;

impl StateMachineTest for ReaderTest {
    type SystemUnderTest = ReaderSut;
    type Reference = ReaderSM;

    fn init_test(ref_status: &<Self::Reference as ReferenceStateMachine>::State) -> Self::SystemUnderTest {
        let buffer = Box::into_raw(Box::new([0; CAP]));
        ReaderSut {
            status: ref_status.clone(),
            buffer,
            producer: DmaMock::default(),
            consumer: ReadableDmaRingBuffer::new(unsafe { &mut *buffer }),
        }
    }

    fn teardown(state: Self::SystemUnderTest) {
        unsafe {
            let _ = Box::from_raw(state.buffer);
        };
    }

    fn apply(
        mut sut: Self::SystemUnderTest,
        ref_state: &<Self::Reference as ReferenceStateMachine>::State,
        transition: <Self::Reference as ReferenceStateMachine>::Transition,
    ) -> Self::SystemUnderTest {
        match transition {
            ReaderTransition::Write(x) => sut.producer.advance(x),
            ReaderTransition::Reset => {
                sut.consumer.reset(&mut sut.producer);
            }
            ReaderTransition::ReadUpTo(x) => {
                let status = sut.status;
                let ReaderSut {
                    ref mut producer,
                    ref mut consumer,
                    ..
                } = sut;
                let mut buf = vec![0; x];
                let res = consumer.read(producer, &mut buf);
                match status {
                    Status::Available(n) => {
                        let readable = x.min(n);

                        assert_eq!(res.unwrap().0, readable);
                    }
                    Status::Failed => assert!(res.is_err()),
                }
            }
        }

        ReaderSut {
            status: ref_state.clone(),
            ..sut
        }
    }
}

prop_state_machine! {
    #[test]
    fn reader_state_test(sequential 1..20 => ReaderTest);
}
