static mut LOGGER: VMWLogger = VMWLogger::new();

pub type Formatter = fn(&mut dyn core::fmt::Write, &log::Record) -> Result<(), core::fmt::Error>;

fn default_formatter(buffer: &mut dyn core::fmt::Write, record: &log::Record) -> Result<(), core::fmt::Error> {
    writeln!(
        buffer,
        "[{}:{} - {}]: {}",
        record.file().unwrap_or("<unknown>"),
        record.line().unwrap_or(0),
        record.level(),
        record.args(),
    )
}

struct VMWLogger {
    formatter: Formatter,
}

impl VMWLogger {
    const fn new() -> Self {
        Self {
            formatter: default_formatter,
        }
    }

    fn set_formatter(&mut self, new_formatter: Formatter) {
        self.formatter = new_formatter;
    }
}

impl core::fmt::Write for VMWLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // credits: jessie
        for c in s.chars() {
            unsafe {
                core::arch::asm!(r#"
                    xchg rbx, {0}
                    mov eax, 0x564D5868
                    mov edx, 0x5658
                    mov ecx, 0x19
                    in eax, dx
                    xchg rbx, {0}
                "#,
                // input is rbx, but llvm is annoying so
                // we just use a temp register and xchg them
                in(reg) c as usize,
                out("rax") _,
                out("rdx") _,
                out("rcx") _,
                );
            }
        }

        Ok(())
    }
}

impl log::Log for VMWLogger {
    fn enabled(&self, _m: &log::Metadata) -> bool {
        // credits: jessie
        unsafe {
            // invokes the 'get_version' command of the vmware backdoor
            // if ebx is not the magic value the backdoor isn't present
            let mut rbx: u64 = 0;
            core::arch::asm!(r#"
                xchg rbx, {_rbx}
                xchg {_rbp}, rbp
                xor rbp, rbp
                mov eax, 0x564D5868
                mov edx, 0x5658
                mov ecx, 0xA
                in eax, dx
                xchg rbp, {_rbp}
                xchg rbx, {_rbx}
            "#,
                in("rax") 0x564D5868,
                _rbx = inout(reg) rbx,
                in("rcx") 0xA,
                in("rdx") 0x5658,
                _rbp = in(reg) 0u64,
                in("rdi") 0,
                in("rsi") 0,
            );
            rbx == 0x564D5868
        }
    }

    fn log(&self, record: &log::Record) {
        let _ = (self.formatter)(&mut VMWLogger::new(), record); // lol
    }

    fn flush(&self) {}
}

pub struct Builder {
    filter: log::LevelFilter,
    formatter: Formatter,
}

impl Builder {
    /// Create a builder for fine-tuning logger.
    pub fn new() -> Self {
        Self {
            filter: log::LevelFilter::Info,
            formatter: default_formatter,
        }
    }

    /// Set the level filter.
    pub fn filter(mut self, filter: log::LevelFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set formatter.
    pub fn formatter(mut self, formatter: Formatter) -> Self {
        self.formatter = formatter;
        self
    }

    /// Setup a logger based on the configuration.
    pub fn setup(self) {
        unsafe {
            LOGGER.set_formatter(self.formatter) ;
            log::set_logger(&LOGGER).unwrap();
        }

        log::set_max_level(self.filter);
    }
}

pub fn builder() -> Builder {
    Builder::new()
}

pub fn init() {
    builder().filter(log::LevelFilter::Info).setup();
}

pub fn init_with_filter(filter: log::LevelFilter) {
    builder().filter(filter).setup();
}