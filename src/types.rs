// Part of ethercat-rs. Copyright 2018-2020 by the authors.
// This work is dual-licensed under Apache 2.0 and MIT terms.

use crate::ec;
use derive_new::new;
use std::{io, os::raw::c_ulong};

pub type Error = io::Error;
pub type Result<T> = io::Result<T>;

pub type MasterIndex = u32;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DomainIndex(pub(crate) usize);

impl From<i32> for DomainIndex {
    fn from(v: i32) -> Self {
        Self(v as usize)
    }
}

impl From<DomainIndex> for c_ulong {
    fn from(idx: DomainIndex) -> Self {
        idx.0 as c_ulong
    }
}

impl From<DomainIndex> for u32 {
    fn from(idx: DomainIndex) -> Self {
        idx.0 as u32
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DomainDataPlacement {
    pub offset: usize,
    pub size: usize,
}

pub type SlaveConfigIndex = u32;
pub type SlavePosition = u16;

/// An EtherCAT slave identification, consisting of vendor ID and product code.
#[derive(Debug, Clone, Copy, new)]
pub struct SlaveId {
    pub vendor_id: u32,
    pub product_code: u32,
}

/// An EtherCAT slave revision identification.
#[derive(Debug, Clone, Copy, new)]
pub struct SlaveRev {
    pub revision_number: u32,
    pub serial_number: u32,
}

/// An EtherCAT slave, which is specified either by absolute position in the
/// ring or by offset from a given alias.
#[derive(Clone, Copy)]
pub enum SlaveAddr {
    ByPos(u16),
    ByAlias(u16, u16),
}

impl SlaveAddr {
    pub(crate) fn as_pair(self) -> (u16, u16) {
        match self {
            SlaveAddr::ByPos(x) => (0, x),
            SlaveAddr::ByAlias(x, y) => (x, y),
        }
    }
}

/// Offset of a PDO entry in the domain image.
#[derive(Debug, Default, PartialEq, Eq, Hash, new)]
pub struct Offset {
    pub byte: usize,
    pub bit: u32,
}

#[derive(Debug, Clone)]
pub struct MasterInfo {
    pub slave_count: u32,
    pub link_up: bool,
    pub scan_busy: bool,
    pub app_time: u64,
}

#[derive(Debug, Clone)]
pub struct MasterState {
    pub slaves_responding: u32,
    pub al_states: u8,
    pub link_up: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigInfo {
    pub alias: u16,
    pub position: u16,
    pub id: SlaveId,
    pub slave_position: Option<u32>,
    pub sdo_count: u32,
    pub idn_count: u32,
    // TODO: more attributes are returned:
    // syncs[*], watchdog_*, dc_*
}

#[derive(Debug, Clone)]
pub struct SlaveInfo {
    pub name: String,
    pub ring_pos: u16,
    pub id: SlaveId,
    pub rev: SlaveRev,
    pub alias: u16,
    pub current_on_ebus: i16,
    pub al_state: AlState,
    pub error_flag: u8,
    pub sync_count: u8,
    pub sdo_count: u16,
    pub ports: [SlavePortInfo; ec::EC_MAX_PORTS as usize],
}

#[derive(Debug, Clone, Copy)]
pub enum SlavePortType {
    NotImplemented,
    NotConfigured,
    EBus,
    MII,
}

impl Default for SlavePortType {
    fn default() -> Self {
        SlavePortType::NotImplemented
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SlavePortLink {
    pub link_up: bool,
    pub loop_closed: bool,
    pub signal_detected: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SlavePortInfo {
    pub desc: SlavePortType,
    pub link: SlavePortLink,
    pub receive_time: u32,
    pub next_slave: u16,
    pub delay_to_next_dc: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SlaveConfigState {
    pub online: bool,
    pub operational: bool,
    pub al_state: AlState,
}

#[derive(Debug, Clone, Copy)]
pub enum SyncDirection {
    Invalid,
    Output,
    Input,
}

#[derive(Debug, Clone, Copy)]
pub enum WatchdogMode {
    Default,
    Enable,
    Disable,
}

pub type SmIndex = u8;
pub type PdoIndex = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, new)]
pub struct PdoEntryIndex {
    pub index: u16,
    pub subindex: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, new)]
pub struct SdoIndex {
    pub index: u16,
    pub subindex: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SdoEntryAddr {
    Position(usize, u8),
    Index(u16, u8),
}

#[derive(Debug, Clone)]
pub struct SdoInfo {
    pub slave_position: u16,
    pub position: u16,
    pub index: u16,
    pub max_subindex: u8,
    pub object_code: u8,
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Access {
    Unknown,
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub struct SdoEntryAccess {
    pub pre_op: Access,
    pub safe_op: Access,
    pub op: Access,
}

impl From<(u8, u8)> for Access {
    fn from((read, write): (u8, u8)) -> Self {
        match (read, write) {
            (0, 1) => Access::ReadOnly,
            (1, 0) => Access::WriteOnly,
            (1, 1) => Access::ReadWrite,
            _ => Access::Unknown,
        }
    }
}

impl From<([u8; 3], [u8; 3])> for SdoEntryAccess {
    fn from((read, write): ([u8; 3], [u8; 3])) -> Self {
        Self {
            pre_op: Access::from((read[0], write[0])),
            safe_op: Access::from((read[1], write[1])),
            op: Access::from((read[2], write[2])),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SdoEntry {
    pub slave_position: u16,
    pub address: SdoEntryAddr,
    pub data_type: DataType,
    pub bit_length: u16,
    pub access: SdoEntryAccess,
    pub description: String,
}

#[derive(Debug, Copy, Clone)]
pub struct SyncInfo<'a> {
    pub index: SmIndex,
    pub direction: SyncDirection,
    pub watchdog_mode: WatchdogMode,
    pub pdos: &'a [PdoInfo<'a>],
}

impl<'a> SyncInfo<'a> {
    pub const fn input(index: SmIndex, pdos: &'a [PdoInfo<'a>]) -> Self {
        SyncInfo {
            index,
            direction: SyncDirection::Input,
            watchdog_mode: WatchdogMode::Default,
            pdos,
        }
    }

    pub const fn output(index: SmIndex, pdos: &'a [PdoInfo<'a>]) -> Self {
        SyncInfo {
            index,
            direction: SyncDirection::Output,
            watchdog_mode: WatchdogMode::Default,
            pdos,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PdoInfo<'a> {
    pub index: PdoIndex,
    pub entries: &'a [PdoEntryInfo],
}

const NO_ENTRIES: &[PdoEntryInfo] = &[];

impl<'a> PdoInfo<'a> {
    pub const fn default(index: PdoIndex) -> PdoInfo<'a> {
        PdoInfo {
            index,
            entries: NO_ENTRIES,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PdoEntryInfo {
    pub index: PdoEntryIndex,
    pub bit_length: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum AlState {
    Init = 1,
    Preop = 2,
    Safeop = 4,
    Op = 8,
}

impl From<u32> for AlState {
    fn from(st: u32) -> Self {
        match st {
            1 => AlState::Init,
            2 => AlState::Preop,
            4 => AlState::Safeop,
            8 => AlState::Op,
            x => panic!("invalid state {}", x),
        }
    }
}

pub trait SdoData {
    fn data_ptr(&self) -> *const u8 {
        self as *const _ as _
    }
    fn data_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

impl SdoData for u8 {}
impl SdoData for u16 {}
impl SdoData for u32 {}
impl SdoData for u64 {}
impl SdoData for i8 {}
impl SdoData for i16 {}
impl SdoData for i32 {}
impl SdoData for i64 {}

impl SdoData for &'_ [u8] {
    fn data_ptr(&self) -> *const u8 {
        self.as_ptr()
    }
    fn data_size(&self) -> usize {
        self.len()
    }
}

#[derive(Debug, Clone)]
pub struct DomainState {
    pub working_counter: u32,
    pub wc_state: WcState,
    pub redundancy_active: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum WcState {
    Zero = 0,
    Incomplete,
    Complete,
}

impl From<u32> for WcState {
    fn from(st: u32) -> Self {
        match st {
            0 => WcState::Zero,
            1 => WcState::Incomplete,
            2 => WcState::Complete,
            x => panic!("invalid state {}", x),
        }
    }
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum DataType {
    Bool = 0x0001,

    I8 = 0x0002,
    I16 = 0x0003,
    I32 = 0x0004,
    U8 = 0x0005,
    U16 = 0x0006,
    U32 = 0x0007,
    F32 = 0x0008,

    String = 0x0009, // a.k.a. visible_string
    OctetString = 0x000A,
    UnicodeString = 0x000B,

    I24 = 0x0010,
    F64 = 0x0011,
    I40 = 0x0012,
    I48 = 0x0013,
    I56 = 0x0014,
    I64 = 0x0015,
    U24 = 0x0016,

    U40 = 0x0018,
    U48 = 0x0019,
    U56 = 0x001A,
    U64 = 0x001B,

    // SM = sign-and-magnitude coding
    Sm8 = 0xFFFB,
    Sm16 = 0xFFFC,
    Sm32 = 0xFFFD,
    Sm64 = 0xFFFE,

    Raw = 0xFFFF,
}

impl DataType {
    /// Number of bytes
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Bool | Self::I8 | Self::U8 | Self::Sm8 => Some(1),
            Self::I16 | Self::U16 | Self::Sm16 => Some(2),
            Self::I24 | Self::U24 => Some(3),
            Self::I32 | Self::U32 | Self::F32 | Self::Sm32 => Some(4),
            Self::I40 | Self::U40 => Some(5),
            Self::I48 | Self::U48 => Some(6),
            Self::I56 | Self::U56 => Some(7),
            Self::I64 | Self::F64 | Self::U64 | Self::Sm64 => Some(8),
            _ => None,
        }
    }
}
