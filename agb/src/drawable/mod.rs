use alloc::boxed::Box;

pub trait AfterVblank {
    fn do_work(&mut self);
}

impl AfterVblank for Box<dyn AfterVblank> {
    fn do_work(&mut self) {
        self.as_mut().do_work();
    }
}

impl AfterVblank for () {
    fn do_work(&mut self) {}
}

impl<T: AfterVblank> AfterVblank for Option<T> {
    fn do_work(&mut self) {
        match self {
            Some(x) => x.do_work(),
            None => {}
        }
    }
}

macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!(T1);
        $name!(T1, T2);
        $name!(T1, T2, T3);
        $name!(T1, T2, T3, T4);
        $name!(T1, T2, T3, T4, T5);
        $name!(T1, T2, T3, T4, T5, T6);
        $name!(T1, T2, T3, T4, T5, T6, T7);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
    };
}

macro_rules! impl_after_vblank {
    ( $($ty:ident),* $(,)? ) => {
        #[allow(non_snake_case)]
        impl<$($ty,)*> AfterVblank for ($($ty,)*)
        where
            $( $ty: AfterVblank, )*
        {
            fn do_work(&mut self) {
                let ($($ty,)*) = self;

                $( $ty.do_work(); )*
            }
        }
    }
}

all_the_tuples!(impl_after_vblank);

pub trait Game {
    type A: AfterVblank;
    fn prepare_frame(&mut self) -> Self::A;
}

pub fn run_game<G: Game>(mut game: G) -> ! {
    let vblank = crate::interrupt::VBlank::get();

    let mut drop_next;
    let mut do_work_later;

    do_work_later = game.prepare_frame();
    vblank.wait_for_vblank();

    do_work_later.do_work();

    drop_next = do_work_later;
    do_work_later = game.prepare_frame();

    loop {
        vblank.wait_for_vblank();
        drop(drop_next);
        do_work_later.do_work();
        drop_next = do_work_later;

        do_work_later = game.prepare_frame();
    }
}
