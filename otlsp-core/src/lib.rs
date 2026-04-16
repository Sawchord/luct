use std::io::ErrorKind;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct OtlspErrorCode(u16);

impl From<ErrorKind> for OtlspErrorCode {
    fn from(error: ErrorKind) -> Self {
        let code = match error {
            ErrorKind::NotFound => 4001,
            ErrorKind::PermissionDenied => 4002,
            ErrorKind::ConnectionRefused => 4003,
            ErrorKind::ConnectionReset => 4004,
            ErrorKind::HostUnreachable => 4005,
            ErrorKind::NetworkUnreachable => 4006,
            ErrorKind::ConnectionAborted => 4007,
            ErrorKind::NotConnected => 4008,
            ErrorKind::AddrInUse => 4009,
            ErrorKind::AddrNotAvailable => 4010,
            ErrorKind::NetworkDown => 4011,
            ErrorKind::BrokenPipe => 4012,
            ErrorKind::AlreadyExists => 4013,
            ErrorKind::WouldBlock => 4014,
            ErrorKind::NotADirectory => 4015,
            ErrorKind::IsADirectory => 4016,
            ErrorKind::DirectoryNotEmpty => 4017,
            ErrorKind::ReadOnlyFilesystem => 4018,
            ErrorKind::StaleNetworkFileHandle => 4019,
            ErrorKind::InvalidInput => 4020,
            ErrorKind::InvalidData => 4021,
            ErrorKind::TimedOut => 4022,
            ErrorKind::WriteZero => 4023,
            ErrorKind::StorageFull => 4024,
            ErrorKind::NotSeekable => 4025,
            ErrorKind::QuotaExceeded => 4026,
            ErrorKind::FileTooLarge => 4027,
            ErrorKind::ResourceBusy => 4028,
            ErrorKind::ExecutableFileBusy => 4029,
            ErrorKind::Deadlock => 4030,
            ErrorKind::CrossesDevices => 4031,
            ErrorKind::TooManyLinks => 4032,
            ErrorKind::InvalidFilename => 4033,
            ErrorKind::ArgumentListTooLong => 4034,
            ErrorKind::Interrupted => 4035,
            ErrorKind::Unsupported => 4036,
            ErrorKind::UnexpectedEof => 4037,
            ErrorKind::OutOfMemory => 4038,
            ErrorKind::Other => 4999,
            _ => 4999,
        };

        Self(code)
    }
}

impl From<OtlspErrorCode> for ErrorKind {
    fn from(code: OtlspErrorCode) -> Self {
        let code = code.0;

        match code {
            4001 => ErrorKind::NotFound,
            4002 => ErrorKind::PermissionDenied,
            4003 => ErrorKind::ConnectionRefused,
            4004 => ErrorKind::ConnectionReset,
            4005 => ErrorKind::HostUnreachable,
            4006 => ErrorKind::NetworkUnreachable,
            4007 => ErrorKind::ConnectionAborted,
            4008 => ErrorKind::NotConnected,
            4009 => ErrorKind::AddrInUse,
            4010 => ErrorKind::AddrNotAvailable,
            4011 => ErrorKind::NetworkDown,
            4012 => ErrorKind::BrokenPipe,
            4013 => ErrorKind::AlreadyExists,
            4014 => ErrorKind::WouldBlock,
            4015 => ErrorKind::NotADirectory,
            4016 => ErrorKind::IsADirectory,
            4017 => ErrorKind::DirectoryNotEmpty,
            4018 => ErrorKind::ReadOnlyFilesystem,
            4019 => ErrorKind::StaleNetworkFileHandle,
            4020 => ErrorKind::InvalidInput,
            4021 => ErrorKind::InvalidData,
            4022 => ErrorKind::TimedOut,
            4023 => ErrorKind::WriteZero,
            4024 => ErrorKind::StorageFull,
            4025 => ErrorKind::NotSeekable,
            4026 => ErrorKind::QuotaExceeded,
            4027 => ErrorKind::FileTooLarge,
            4028 => ErrorKind::ResourceBusy,
            4029 => ErrorKind::ExecutableFileBusy,
            4030 => ErrorKind::Deadlock,
            4031 => ErrorKind::CrossesDevices,
            4032 => ErrorKind::TooManyLinks,
            4033 => ErrorKind::InvalidFilename,
            4034 => ErrorKind::ArgumentListTooLong,
            4035 => ErrorKind::Interrupted,
            4036 => ErrorKind::Unsupported,
            4037 => ErrorKind::UnexpectedEof,
            4038 => ErrorKind::OutOfMemory,
            _ => ErrorKind::Other,
        }
    }
}

impl From<u16> for OtlspErrorCode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<OtlspErrorCode> for u16 {
    fn from(value: OtlspErrorCode) -> Self {
        value.0
    }
}
