#![deny(unsafe_op_in_unsafe_fn)]
use libc::c_void;
use mgba_sys::{mCPUComponent, ARMCore};

#[derive(Default)]
enum ArmInnerComponentParameters<T: CpuComponent> {
    #[default]
    Blank,
    Parameters(T::InitParameters),
    Componennt(T),
}

#[repr(C)]
struct ArmCpuComponentInner<T: CpuComponent> {
    m_component: mgba_sys::mCPUComponent,
    inner: ArmInnerComponentParameters<T>,
}

pub struct ArmCpuComponent<T: CpuComponent> {
    inner: Box<ArmCpuComponentInner<T>>,
}

impl<T: CpuComponent> ArmCpuComponentInner<T> {
    fn new(init: T::InitParameters) -> Self {
        Self {
            m_component: generate_component::<T>(),
            inner: ArmInnerComponentParameters::Parameters(init),
        }
    }
}

impl<T: CpuComponent> ArmCpuComponent<T> {
    pub fn new(init: T::InitParameters) -> Self {
        let b = Box::new(ArmCpuComponentInner::new(init));

        Self { inner: b }
    }

    fn into_mgba(self) -> *mut mgba_sys::mCPUComponent {
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

unsafe extern "C" fn init_component<T: CpuComponent>(cpu: *mut c_void, this: *mut mCPUComponent) {
    let cpu: *mut mgba_sys::ARMCore = cpu.cast();
    let this: *mut ArmCpuComponentInner<T> = this.cast();

    // safety: we have a pointer to ourself that is valid by this functions guarentees
    let inner = core::mem::take(unsafe { &mut (*this).inner });
    let component = match inner {
        // safety: we can new up because we're the only ones here right now
        ArmInnerComponentParameters::Parameters(params) => unsafe { T::new(cpu, params) },
        _ => panic!("invalid state"),
    };

    unsafe { (*this).inner = ArmInnerComponentParameters::Componennt(component) };
}

extern "C" fn deinit_component<T: CpuComponent>(this: *mut mCPUComponent) {
    let this: *mut ArmCpuComponentInner<T> = this.cast();

    drop(unsafe { Box::from_raw(this) });
}

/// # Note to implementors.
/// You should have a drop that cleans up after yourself, restoring the cpu to a
/// good state. As your drop won't be called with the arm core, you should store
/// the arm core in order to clean it up.
///
/// # Sealed
/// * Note that this trait is sealed so can't be implemented externally.
///
/// # Safety
/// * The ID for your trait should be *globally* unique.
pub unsafe trait CpuComponent: sealed::Sealed {
    const ID: u32;

    type InitParameters;

    /// # Safety
    /// * You must call this function with a valid pointer to an mgba arm core.
    /// * You must call the drop for this before newing up ANY OTHER component.
    unsafe fn new(cpu: *mut mgba_sys::ARMCore, parameters: Self::InitParameters) -> Self;
}

mod sealed {
    pub trait Sealed {}
    impl<T: super::CpuComponent> Sealed for super::Handshake<T> {}
    impl Sealed for super::InterruptCatcher {}
}

unsafe fn find_self<T: CpuComponent>(cpu: *mut ARMCore) -> Option<*mut T> {
    unsafe {
        let num_components = (*cpu).numComponents;

        for idx in 0..num_components {
            let component = ((*cpu).components).add(idx);
            if !((*component).is_null()) && (**component).id == T::ID {
                let inner_component: *mut ArmCpuComponentInner<T> = (*component).cast();
                let inner = &mut *inner_component;

                let component = match &mut inner.inner {
                    ArmInnerComponentParameters::Componennt(a) => a,
                    _ => panic!("should be a component at this point"),
                };
                return Some(component);
            }
        }
    }

    None
}

pub struct ArmCore(*mut ARMCore);

impl ArmCore {
    pub fn scratch_registers(&self) -> [u32; 4] {
        let mut destination: [u32; 4] = Default::default();
        for (&reg, dest) in unsafe { &(*self.0).__bindgen_anon_1.__bindgen_anon_1.gprs[0..4] }
            .iter()
            .zip(destination.iter_mut())
        {
            *dest = reg as u32;
        }
        destination
    }

    pub fn set_scratch_registers(&mut self, scratch_registers: [u32; 4]) {
        for (reg, &next_reg) in
            unsafe { &mut (*self.0).__bindgen_anon_1.__bindgen_anon_1.gprs[0..4] }
                .iter_mut()
                .zip(scratch_registers.iter())
        {
            *reg = next_reg as i32;
        }
    }

    pub fn view_32(&mut self, address: u32) -> u32 {
        unsafe { mgba_sys::GBAView32(self.0, address) }
    }

    pub fn patch_32(&mut self, address: u32, value: u32) {
        unsafe { mgba_sys::GBAPatch32(self.0, address, value as i32, std::ptr::null_mut()) }
    }
}

pub struct Handshake<T: CpuComponent> {
    cpu: *mut ARMCore,
    old_cpu_memory: mgba_sys::ARMMemory,
    has_written_expected_value: bool,
    init: HandshakeInit<T::InitParameters>,
}

pub struct HandshakeInit<T> {
    address: u32,
    expected_write: i32,
    returned_read: u32,
    inner_init: Option<T>,
}

impl<T> HandshakeInit<T> {
    pub const fn new(address: u32, expected_write: u32, returned_read: u32, inner_init: T) -> Self {
        Self {
            address,
            expected_write: expected_write as i32,
            returned_read,
            inner_init: Some(inner_init),
        }
    }
}

const OUR_SLOT: usize = mgba_sys::mCPUComponentType_CPU_COMPONENT_MISC_4 as usize;

pub unsafe fn plugin_component<T: CpuComponent>(
    cpu: *mut ARMCore,
    plugin_component: ArmCpuComponent<T>,
) {
    // safety: cpu is an initialised ARMCore, our add is within range of the array
    let component: *mut *mut mgba_sys::mCPUComponent = unsafe { ((*cpu).components).add(OUR_SLOT) };

    // safety: pointer is in array, we're checking for whether it's null and iff
    // non null we call the C function that cleans up
    unsafe {
        if !(*component).is_null() {
            mgba_sys::ARMHotplugDetach(cpu, OUR_SLOT);
        }
    }

    // safety: again, we know the pointer is good. We're setting the pointer to
    // the object wanted by the hotplug attach function
    unsafe {
        (*component) = plugin_component.into_mgba();
        mgba_sys::ARMHotplugAttach(cpu, OUR_SLOT);
    }
}

unsafe extern "C" fn read_32_intercept<T: CpuComponent>(
    cpu: *mut ARMCore,
    address: u32,
    cycle_counter: *mut i32,
) -> u32 {
    // safety: cpu is valid pointer
    let Some(this) = (unsafe { find_self::<Handshake<T>>(cpu) }) else {
        return 0;
    };

    // safety: our this pointer is valid for this type at this point
    let this = unsafe { &mut *this };

    if this.has_written_expected_value && this.init.address == address {
        let read = this.init.returned_read;
        let init = this.init.inner_init.take().unwrap();

        // safety: after calling this, we cannot refer to anything from our `this` pointer again, and we don't.
        unsafe {
            plugin_component(cpu, ArmCpuComponent::<T>::new(init));
        }

        read
    } else {
        // safety: can call mgba function (it was going to be called anyway)
        unsafe { this.old_cpu_memory.load32.unwrap()(cpu, address, cycle_counter) }
    }
}

unsafe extern "C" fn write_32_intercept<T: CpuComponent>(
    cpu: *mut ARMCore,
    address: u32,
    value: i32,
    cycle_counter: *mut i32,
) {
    let this_ptr = (unsafe { find_self::<Handshake<T>>(cpu) })
        .expect("we should be able to find our handshake for while the writer function is present");

    // safety: our this pointer is valid for this type at this point
    let this = unsafe { &mut *this_ptr };

    if this.init.address == address && this.init.expected_write == value {
        this.has_written_expected_value = true;
    } else {
        // safety: can call mgba function (it was going to be called anyway)

        unsafe {
            this.old_cpu_memory
                .store32
                .expect("the store32 function should exist")(
                cpu, address, value, cycle_counter
            );
        }
    }
}

impl<T: CpuComponent> Drop for Handshake<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.cpu).memory.load32 = self.old_cpu_memory.load32;
            (*self.cpu).memory.store32 = self.old_cpu_memory.store32;
        };
    }
}

unsafe impl<T: CpuComponent> CpuComponent for Handshake<T> {
    const ID: u32 = 99;

    type InitParameters = HandshakeInit<T::InitParameters>;

    unsafe fn new(cpu: *mut mgba_sys::ARMCore, init: Self::InitParameters) -> Self {
        unsafe {
            let old_memory = (*cpu).memory;

            (*cpu).memory.load32 = Some(read_32_intercept::<T>);
            (*cpu).memory.store32 = Some(write_32_intercept::<T>);

            Handshake {
                cpu,
                old_cpu_memory: old_memory,
                has_written_expected_value: false,
                init,
            }
        }
    }
}

pub struct InterruptCatcher {
    cpu: *mut mgba_sys::ARMCore,
    old_cpu_interrupts: mgba_sys::ARMInterruptHandler,
    init: InterruptCatcherInit,
}

impl Drop for InterruptCatcher {
    fn drop(&mut self) {
        unsafe {
            (*self.cpu).irqh = self.old_cpu_interrupts;
        }
    }
}

unsafe extern "C" fn swi16(cpu: *mut ARMCore, immediate: i32) {
    // safety: cpu is valid pointer
    let this = unsafe { find_self::<InterruptCatcher>(cpu) }
        .expect("for swi function to be active, interrupt handler needs to be active");

    let mut core = ArmCore(cpu);

    // we need to be careful here as the cpu and this refer to eachother a fair
    // amount, interacting via raw pointers should be okay

    // can dereference this as it is valid from guarentee of find_self.
    let on_swi = unsafe { &(*this).init.on_swi };

    if !on_swi(&mut core, immediate as u32) {
        // safety: can call regular mgba function as it would be called anyway
        unsafe {
            (*this)
                .old_cpu_interrupts
                .swi16
                .inspect(|x| x(cpu, immediate));
        }
    }
}

pub struct InterruptCatcherInit {
    on_swi: Box<dyn Fn(&mut ArmCore, u32) -> bool>,
}

impl InterruptCatcherInit {
    /// This will be called for each interrupt, if you return true then the
    /// normal interrupts will be skipped, otherwise normal interrupts will be
    /// called.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut ArmCore, u32) -> bool + 'static,
    {
        let b = Box::new(f);

        Self { on_swi: b }
    }
}

unsafe impl CpuComponent for InterruptCatcher {
    const ID: u32 = 42;

    type InitParameters = InterruptCatcherInit;

    unsafe fn new(cpu: *mut mgba_sys::ARMCore, init: Self::InitParameters) -> Self {
        unsafe {
            let old_irqs = (*cpu).irqh;
            (*cpu).irqh.swi16 = Some(swi16);
            Self {
                cpu,
                old_cpu_interrupts: old_irqs,
                init,
            }
        }
    }
}
