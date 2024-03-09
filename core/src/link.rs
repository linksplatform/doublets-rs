use {
    crate::LinkType,
    std::ops::{ControlFlow, FromResidual, Try},
};

#[repr(usize)]
pub enum Link<T: LinkType> {
    Any,
    ItSelf,
    Link(T),
}

#[repr(usize)]
pub enum Flow {
    Continue,
    Break,
}

impl FromResidual for Flow {
    fn from_residual(_: <Self as Try>::Residual) -> Self {
        Flow::Break
    }
}

impl Try for Flow {
    type Output = ();
    type Residual = Flow;

    fn from_output(_: Self::Output) -> Self {
        Flow::Continue
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Flow::Continue => ControlFlow::Continue(()),
            Flow::Break => ControlFlow::Break(Flow::Break),
        }
    }
}

impl<C, B> From<ControlFlow<C, B>> for Flow {
    fn from(flow: ControlFlow<C, B>) -> Self {
        match flow {
            ControlFlow::Continue(_) => Flow::Continue,
            ControlFlow::Break(_) => Flow::Break,
        }
    }
}
