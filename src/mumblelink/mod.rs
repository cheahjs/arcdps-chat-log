use std::{io, mem, ptr};

use anyhow::bail;
use bitflags::bitflags;
use log::debug;
use winapi::ctypes::{c_void, wchar_t};

struct LinkHandle(*mut c_void);
unsafe impl Send for LinkHandle {}
unsafe impl Sync for LinkHandle {}

bitflags! {
    #[repr(C)]
    pub struct UiState: u32 {
        const IS_MAP_OPEN = 0b1;
        const IS_COMPASS_TOP_RIGHT = 0b10;
        const DOES_COMPASS_HAVE_ROTATION_ENABLED = 0b100;
        const GAME_HAS_FOCUS = 0b1000;
        const IS_IN_COMPETITIVE_MODE = 0b10000;
        const TEXTBOX_HAS_FOCUS = 0b100000;
        const IS_IN_COMBAT = 0b1000000;
    }
}

#[repr(C)]
pub struct Context {
    pub server_address: [u8; 28],
    pub map_id: u32,
    pub map_type: u32,
    pub shard_id: u32,
    pub instance: u32,
    pub build_id: u32,
    pub ui_state: UiState,
    pub compass_width: u16,
    pub compass_height: u16,
    pub compass_rotation: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub map_center_x: f32,
    pub map_center_y: f32,
    pub map_scale: f32,
    pub process_id: u32,
    pub mount_index: u8,
}

impl Context {
    pub fn is_in_combat(&self) -> bool {
        self.ui_state.contains(UiState::IS_IN_COMBAT)
    }

    #[allow(dead_code)]
    pub fn game_has_focus(&self) -> bool {
        self.ui_state.contains(UiState::GAME_HAS_FOCUS)
    }
}

#[repr(C)]
pub struct Position {
    /// The character's position in space.
    pub position: [f32; 3],
    /// A unit vector pointing out of the character's eyes.
    pub front: [f32; 3],
    /// A unit vector pointing out of the top of the character's head.
    pub top: [f32; 3],
}

#[repr(C)]
pub struct LinkedMem {
    pub ui_version: u32,
    pub ui_tick: u32,
    pub avatar: Position,
    pub name: [wchar_t; 256],
    pub camera: Position,
    pub identity: [wchar_t; 256],
    pub context_len: u32,
    pub context: Context,
    pub description: [wchar_t; 2048],
}

pub struct MumbleLink {
    handle: Option<LinkHandle>,
    linked_mem: Option<LinkHandle>,
}

impl MumbleLink {
    pub fn new() -> Self {
        Self {
            handle: None,
            linked_mem: None,
        }
    }

    pub fn load(&mut self) -> anyhow::Result<String> {
        let linked_mem_size = mem::size_of::<LinkedMem>();
        let mumble_link_name = Self::get_mumble_link_name();
        if mumble_link_name == "0" {
            bail!("mumblelink is disabled");
        }
        debug!("mumblelink name: {}", mumble_link_name);
        let mut shared_file: Vec<wchar_t> =
            mumble_link_name.chars().map(|c| c as wchar_t).collect();
        shared_file.push(0);

        unsafe {
            let handle = kernel32::CreateFileMappingW(
                winapi::um::handleapi::INVALID_HANDLE_VALUE,
                ptr::null_mut(),
                winapi::um::winnt::PAGE_READWRITE,
                0,
                linked_mem_size as u32,
                shared_file.as_ptr(),
            );
            if handle.is_null() {
                return Err(io::Error::last_os_error().into());
            }

            let pointer = kernel32::MapViewOfFile(
                handle,
                winapi::um::memoryapi::FILE_MAP_READ,
                0,
                0,
                linked_mem_size as u64,
            );
            if pointer.is_null() {
                kernel32::CloseHandle(handle);
                return Err(io::Error::last_os_error().into());
            }

            self.handle = Some(LinkHandle(handle));
            self.linked_mem = Some(LinkHandle(pointer));
            Ok(mumble_link_name)
        }
    }

    pub fn tick(&mut self) -> Option<LinkedMem> {
        if let Some(linked_mem) = self.linked_mem.as_ref() {
            unsafe {
                return Some(ptr::read_volatile(linked_mem.0 as *const LinkedMem));
            }
        }
        None
    }

    fn get_mumble_link_name() -> String {
        std::env::args()
            .skip_while(|arg| arg != "-mumble")
            .nth(1)
            .unwrap_or_else(|| "MumbleLink".to_string())
    }
}

impl Drop for MumbleLink {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.as_ref() {
            unsafe {
                kernel32::CloseHandle(handle.0);
            }
        }
    }
}
