use failure::Fail;
use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Invalid instruction: {}", _0)]
    InvalidInstruction(InvalidInstructionCtx),

    #[fail(display = "Stack overflow")]
    StackOverflow,

    #[fail(display = "Stack underflow")]
    StackUnderflow,

    #[fail(display = "Invalid address 0x{:016x}", _0)]
    InvalidAddress(u64),

    #[fail(display = "Invalid function ID {}", _0)]
    InvalidFnId(u32),

    #[fail(display = "Invalid instruction offset")]
    InvalidInstructionOffset,

    #[fail(display = "Dividing by zero")]
    DivZero,

    #[fail(display = "Arithmetic error")]
    ArithmeticErr,

    #[fail(display = "Out of memory")]
    OutOfMemory,

    #[fail(display = "Unaligned memory access of address 0x{:016x}", _0)]
    UnalignedAccess(u64),

    #[fail(display = "Control reaches end of function #{} without returning", _0)]
    ControlReachesEnd(u32),

    #[fail(display = "Unable to find entry point")]
    NoEntryPoint,

    #[fail(display = "IO error")]
    IoError(std::io::Error),

    #[fail(display = "Halt")]
    Halt,
}

#[derive(Debug)]
pub struct InvalidInstructionCtx {
    /// Instruction opcode
    pub inst: u8,
    /// Function id
    pub fn_id: u32,
    /// Instruction offset
    pub inst_off: u64,
}

impl Display for InvalidInstructionCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:02x} at fn #{}:{}",
            self.inst, self.fn_id, self.inst_off
        )
    }
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Error::IoError(x)
    }
}
