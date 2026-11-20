// Copyright (c) 2026 AmiCradle Team. MIT.
//! Safe MIDI-facing binding to `gfopensynth.library` version 1.

#![no_std]

#[cfg(target_arch = "m68k")]
use amiga_runtime::{OpenLibrary, Runtime};
#[cfg(target_arch = "m68k")]
use core::ffi::c_void;
#[cfg(target_arch = "m68k")]
use core::ptr::NonNull;

const STATUS_NOTE_OFF: u8 = 0x80;
const STATUS_NOTE_ON: u8 = 0x90;
const STATUS_CONTROL: u8 = 0xb0;
const STATUS_PROGRAM: u8 = 0xc0;
const STATUS_PITCH_BEND: u8 = 0xe0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageError {
    InvalidChannel,
    InvalidData,
    InvalidPitchBend,
    DataTooLong,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShortMessage(u32);

impl ShortMessage {
    pub fn note_on(channel: u8, pitch: u8, velocity: u8) -> Result<Self, MessageError> {
        pack_channel_message(STATUS_NOTE_ON, channel, pitch, velocity)
    }

    pub fn note_off(channel: u8, pitch: u8, release_velocity: u8) -> Result<Self, MessageError> {
        pack_channel_message(STATUS_NOTE_OFF, channel, pitch, release_velocity)
    }

    pub fn control_change(channel: u8, controller: u8, value: u8) -> Result<Self, MessageError> {
        pack_channel_message(STATUS_CONTROL, channel, controller, value)
    }

    pub fn program_change(channel: u8, program: u8) -> Result<Self, MessageError> {
        validate_channel(channel)?;
        validate_data(program)?;
        Ok(Self(
            u32::from(STATUS_PROGRAM | channel) | (u32::from(program) << 8),
        ))
    }

    pub fn pitch_bend(channel: u8, value: i16) -> Result<Self, MessageError> {
        validate_channel(channel)?;
        if !(-8_192..=8_191).contains(&value) {
            return Err(MessageError::InvalidPitchBend);
        }
        let raw =
            u16::try_from(i32::from(value) + 8_192).map_err(|_| MessageError::InvalidPitchBend)?;
        let least = u8::try_from(raw & 0x7f).map_err(|_| MessageError::InvalidPitchBend)?;
        let most = u8::try_from(raw >> 7).map_err(|_| MessageError::InvalidPitchBend)?;
        pack_channel_message(STATUS_PITCH_BEND, channel, least, most)
    }

    pub fn all_notes_off(channel: u8) -> Result<Self, MessageError> {
        Self::control_change(channel, 123, 0)
    }

    pub const fn packed(self) -> u32 {
        self.0
    }
}

fn pack_channel_message(
    status: u8,
    channel: u8,
    first: u8,
    second: u8,
) -> Result<ShortMessage, MessageError> {
    validate_channel(channel)?;
    validate_data(first)?;
    validate_data(second)?;
    Ok(ShortMessage(
        u32::from(status | channel) | (u32::from(first) << 8) | (u32::from(second) << 16),
    ))
}

fn validate_channel(channel: u8) -> Result<(), MessageError> {
    if channel < 16 {
        Ok(())
    } else {
        Err(MessageError::InvalidChannel)
    }
}

fn validate_data(value: u8) -> Result<(), MessageError> {
    if value < 128 {
        Ok(())
    } else {
        Err(MessageError::InvalidData)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenError {
    LibraryUnavailable,
    SynthUnavailable,
}

#[cfg(target_arch = "m68k")]
const LVO_OPEN_SYNTH_A: i32 = -0x1e;
#[cfg(target_arch = "m68k")]
const LVO_CLOSE_SYNTH: i32 = -0x24;
#[cfg(target_arch = "m68k")]
const LVO_SHORT_MSG: i32 = -0x2a;
#[cfg(target_arch = "m68k")]
const LVO_SYSEX: i32 = -0x30;
#[cfg(target_arch = "m68k")]
const LVO_RESET: i32 = -0x48;

/// An open synthesiser using GFOpenSynth's automatic output selection.
///
/// AkikoPlus's MIDI port is preferred by the library.  When that device is
/// absent, the library's documented 68k/AHI fallback applies.
#[cfg(target_arch = "m68k")]
pub struct Synth<'runtime> {
    library: OpenLibrary<'runtime>,
    synth: NonNull<c_void>,
}

#[cfg(target_arch = "m68k")]
impl<'runtime> Synth<'runtime> {
    pub fn open(runtime: &'runtime Runtime) -> Result<Self, OpenError> {
        let library = runtime
            .open_library(c"gfopensynth.library", 1)
            .ok_or(OpenError::LibraryUnavailable)?;
        let raw = unsafe { library.call_a0(LVO_OPEN_SYNTH_A, core::ptr::null_mut()) } as usize
            as *mut c_void;
        let synth = NonNull::new(raw).ok_or(OpenError::SynthUnavailable)?;
        Ok(Self { library, synth })
    }

    pub fn send_short(&mut self, message: ShortMessage) {
        unsafe {
            self.library
                .call_a0_d0(LVO_SHORT_MSG, self.synth.as_ptr(), message.packed());
        }
    }

    pub fn send_sysex(&mut self, bytes: &[u8]) -> Result<(), MessageError> {
        let length = u32::try_from(bytes.len()).map_err(|_| MessageError::DataTooLong)?;
        unsafe {
            self.library.call_a0_a1_d0(
                LVO_SYSEX,
                self.synth.as_ptr(),
                bytes.as_ptr().cast_mut().cast(),
                length,
            );
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        unsafe {
            self.library.call_a0(LVO_RESET, self.synth.as_ptr());
        }
    }
}

#[cfg(target_arch = "m68k")]
impl Drop for Synth<'_> {
    fn drop(&mut self) {
        unsafe {
            self.library.call_a0(LVO_CLOSE_SYNTH, self.synth.as_ptr());
        }
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_messages_match_gfopensynth_packing() {
        assert_eq!(
            ShortMessage::note_on(2, 60, 100).map(ShortMessage::packed),
            Ok(0x0064_3c92)
        );
        assert_eq!(
            ShortMessage::note_off(2, 60, 64).map(ShortMessage::packed),
            Ok(0x0040_3c82)
        );
        assert_eq!(
            ShortMessage::control_change(0, 7, 127).map(ShortMessage::packed),
            Ok(0x007f_07b0)
        );
        assert_eq!(
            ShortMessage::program_change(15, 127).map(ShortMessage::packed),
            Ok(0x0000_7fcf)
        );
        assert_eq!(
            ShortMessage::all_notes_off(3).map(ShortMessage::packed),
            Ok(0x0000_7bb3)
        );
    }

    #[test]
    fn pitch_bend_is_centred_and_bounded() {
        assert_eq!(
            ShortMessage::pitch_bend(0, 0).map(ShortMessage::packed),
            Ok(0x0040_00e0)
        );
        assert_eq!(
            ShortMessage::pitch_bend(0, -8_192).map(ShortMessage::packed),
            Ok(0x0000_00e0)
        );
        assert_eq!(
            ShortMessage::pitch_bend(0, 8_191).map(ShortMessage::packed),
            Ok(0x007f_7fe0)
        );
        assert_eq!(
            ShortMessage::pitch_bend(0, 8_192),
            Err(MessageError::InvalidPitchBend)
        );
    }

    #[test]
    fn malformed_midi_values_are_rejected() {
        assert_eq!(
            ShortMessage::note_on(16, 60, 100),
            Err(MessageError::InvalidChannel)
        );
        assert_eq!(
            ShortMessage::note_on(0, 128, 100),
            Err(MessageError::InvalidData)
        );
        assert_eq!(
            ShortMessage::note_on(0, 60, 128),
            Err(MessageError::InvalidData)
        );
    }
}
