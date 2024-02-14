use libc::c_void;
use mgba_sys::{mCPUComponent, ARMCore};

#[repr(C)]
struct ArmCpuComponentInner<T> {
    m_component: mgba_sys::mCPUComponent,
    inner: Option<T>,
}

pub struct ArmCpuComponent<T> {
    inner: Box<ArmCpuComponentInner<T>>,
}

impl<T: CpuComponent> ArmCpuComponentInner<T> {
    fn new() -> Self {
        Self {
            m_component: generate_component::<T>(),
            inner: None,
        }
    }
}

impl<T: CpuComponent> ArmCpuComponent<T> {
    pub fn new() -> Self {
        let b = Box::new(ArmCpuComponentInner::new());

        Self { inner: b }
    }

    pub(crate) fn into_mgba(self) -> *mut mgba_sys::mCPUComponent {
        Box::into_raw(self.inner).cast()
    }
}

fn generate_component<T: CpuComponent>() -> mgba_sys::mCPUComponent {
    mgba_sys::mCPUComponent {
        id: T::ID,
        init: Some(init_component::<T>),
        deinit: Some(deinit_component::<T>),
    }
}

extern "C" fn init_component<T: CpuComponent>(cpu: *mut c_void, this: *mut mCPUComponent) {
    let cpu: *mut mgba_sys::ARMCore = cpu.cast();
    let this: *mut ArmCpuComponentInner<T> = this.cast();
    unsafe { (*this).inner = Some(T::new(cpu)) }
}

extern "C" fn deinit_component<T: CpuComponent>(this: *mut mCPUComponent) {
    let this: *mut ArmCpuComponentInner<T> = this.cast();

    drop(unsafe { Box::from_raw(this) });
}

pub unsafe trait CpuComponent {
    const ID: u32;

    fn new(cpu: *mut mgba_sys::ARMCore) -> Self;
}

unsafe fn find_self<T: CpuComponent>(cpu: *mut ARMCore) -> Option<*mut T> {
    unsafe {
        let num_components = (*cpu).numComponents;

        for idx in 0..num_components {
            let component = ((*cpu).components).add(idx);
            if !((*component).is_null()) && (**component).id == T::ID {
                return Some(component.cast());
            }
        }
    }

    None
}

pub struct InterruptCatcher {
    cpu: *mut mgba_sys::ARMCore,
    old_cpu_interrupts: mgba_sys::ARMInterruptHandler,
}

unsafe extern "C" fn swi16(cpu: *mut ARMCore, immediate: i32) {
    println!("SWI16 {:0x}", immediate);

    let this = find_self::<InterruptCatcher>(cpu).unwrap();

    (*this)
        .old_cpu_interrupts
        .swi16
        .inspect(|x| x(cpu, immediate));
}

impl Drop for InterruptCatcher {
    fn drop(&mut self) {
        unsafe {
            (*self.cpu).irqh = self.old_cpu_interrupts;
        }
    }
}

unsafe impl CpuComponent for InterruptCatcher {
    const ID: u32 = 42;

    fn new(cpu: *mut mgba_sys::ARMCore) -> Self {
        unsafe {
            let old_irqs = (*cpu).irqh;
            (*cpu).irqh.swi16 = Some(swi16);
            Self {
                cpu,
                old_cpu_interrupts: old_irqs,
            }
        }
    }
}
