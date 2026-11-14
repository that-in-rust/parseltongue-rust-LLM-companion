#!/usr/bin/env python3
"""Build the source-only occurrence-separated itoa call-site inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

SOURCE_HASHES = {
    "src/lib.rs": "e962338e1886873aa7c3399ec89aa5378ccbfed712b9690f043628079390a34f",
    "src/u128_ext.rs": "498cc94f13805e551248fd82f47cc318f51d953a5002a9f8698858cd12fbae3c",
}
ALLOWED_TARGET_KEY_TYPES = {"SemanticOccurrenceKeyValue", "NameSymbolTargetKey::Builtin"}

INTEGER_PAIRS = [
    ("i8", "u8", "invoke_integer_i8_u8"),
    ("i16", "u16", "invoke_integer_i16_u16"),
    ("i32", "u32", "invoke_integer_i32_u32"),
    ("i64", "u64", "invoke_integer_i64_u64"),
    ("i128", "u128", "invoke_integer_i128_u128"),
]
POINTER_PAIRS = [
    ("isize", "i64", "invoke_pointer_isize"),
    ("usize", "u64", "invoke_pointer_usize"),
]
UNSIGNED_RECEIVERS = [
    ("u8", "invoke_unsigned_u8"),
    ("u16", "invoke_unsigned_u16"),
    ("u32", "invoke_unsigned_u32"),
    ("u64", "invoke_unsigned_u64"),
]
CALL_TEMPLATES: list[dict] = [
    {"id":"decimal-pairs-static-constructor","group":"single","owner":"itoa::DECIMAL_PAIRS","owner_kind":"static_initializer","target":"itoa::DecimalPairs","target_kind":"local","file":"src/lib.rs","lines":[222,228],"expression":"DecimalPairs(","token":"DecimalPairs","dispatch":"tuple_struct_constructor_candidate","target_status":"constructor_specific_semantic_target_key_unverified"},
    {"id":"default-new","group":"single","owner":"<itoa::Buffer as core::default::Default>::default","target":"itoa::Buffer::new","target_kind":"local","file":"src/lib.rs","lines":[79,79],"expression":"Buffer::new()","token":"Buffer::new","dispatch":"ordinary_local_call","target_status":"local_target_expected_occurrence_key_pending"},
    {"id":"clone-new","group":"single","owner":"<itoa::Buffer as core::clone::Clone>::clone","target":"itoa::Buffer::new","target_kind":"local","file":"src/lib.rs","lines":[89,89],"expression":"Buffer::new()","token":"Buffer::new","dispatch":"ordinary_local_call","target_status":"local_target_expected_occurrence_key_pending"},
    {"id":"new-uninit","group":"single","owner":"itoa::Buffer::new","target":"core::mem::MaybeUninit::uninit","target_kind":"builtin","file":"src/lib.rs","lines":[99,99],"expression":"MaybeUninit::<u8>::uninit()","token":"uninit","dispatch":"builtin_associated_function","target_status":"builtin_key_target_unverified"},
    {"id":"format-as-mut-ptr","group":"single","owner":"itoa::Buffer::format","target":"core::slice::<T>::as_mut_ptr","target_kind":"builtin","file":"src/lib.rs","lines":[107,107],"expression":"self.bytes.as_mut_ptr()","token":"as_mut_ptr","dispatch":"builtin_method_autoderef","target_status":"builtin_method_target_unverified"},
    {"id":"format-pointer-cast","group":"single","owner":"itoa::Buffer::format","target":"core::ptr::mut_ptr::<T>::cast","target_kind":"builtin","file":"src/lib.rs","lines":[107,107],"expression":".cast::<I::Buffer>()","token":"cast","dispatch":"builtin_pointer_method","target_status":"builtin_method_target_unverified"},
    {"id":"format-generic-sealed-write","group":"single","owner":"itoa::Buffer::format","target":"itoa::private::Sealed::write","target_kind":"local","file":"src/lib.rs","lines":[108,108],"expression":"i.write(unsafe { &mut *buf_ptr })","token":"write","dispatch":"generic_trait_declaration","target_status":"generic_trait_declaration_no_concrete_fanout"},
    {"id":"format-str-len","group":"single","owner":"itoa::Buffer::format","target":"core::primitive::str::len","target_kind":"builtin","file":"src/lib.rs","lines":[109,109],"expression":"string.len()","token":"len","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"format-unreachable-unchecked","group":"single","owner":"itoa::Buffer::format","target":"core::hint::unreachable_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[110,110],"expression":"hint::unreachable_unchecked()","token":"unreachable_unchecked","dispatch":"builtin_function","target_status":"builtin_key_target_unverified"},
    {"id":"max-len-unsigned-ilog10","group":"integer_unsigned","owner":"<core::primitive::{receiver} as itoa::Integer>::MAX_STR_LEN","target":"core::primitive::{receiver}::ilog10","target_kind":"builtin","file":"src/lib.rs","lines":[144,144],"expression":"$Unsigned::MAX.ilog10()","token":"ilog10","dispatch":"primitive_method_concrete_receiver","target_status":"builtin_primitive_target_unverified"},
    {"id":"max-len-signed-ilog10","group":"integer_signed","owner":"<core::primitive::{receiver} as itoa::Integer>::MAX_STR_LEN","target":"core::primitive::{receiver}::ilog10","target_kind":"builtin","file":"src/lib.rs","lines":[160,160],"expression":"$Signed::MAX.ilog10()","token":"ilog10","dispatch":"primitive_method_concrete_receiver","target_status":"builtin_primitive_target_unverified"},
    {"id":"sealed-unsigned-fmt","group":"sealed_unsigned","owner":"<core::primitive::{receiver} as itoa::private::Sealed>::write","target":"<core::primitive::{unsigned} as itoa::Unsigned>::fmt","target_kind":"local","file":"src/lib.rs","lines":[153,153],"expression":"Unsigned::fmt(self, buf)","token":"Unsigned::fmt","dispatch":"concrete_trait_impl_expectation","target_status":"source_concrete_Unsigned_impl_target_requires_independent_verification"},
    {"id":"sealed-unsigned-slice","group":"sealed_unsigned","owner":"<core::primitive::{receiver} as itoa::private::Sealed>::write","target":"itoa::slice_buffer_to_str","target_kind":"local","file":"src/lib.rs","lines":[155,155],"expression":"slice_buffer_to_str(buf, offset)","token":"slice_buffer_to_str","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"sealed-signed-fmt","group":"sealed_signed","owner":"<core::primitive::{signed} as itoa::private::Sealed>::write","target":"<core::primitive::{unsigned} as itoa::Unsigned>::fmt","target_kind":"local","file":"src/lib.rs","lines":[170,173],"expression":"Unsigned::fmt(self.unsigned_abs(), (&mut buf[offset..]).try_into().unwrap())","token":"Unsigned::fmt","dispatch":"concrete_trait_impl_expectation","target_status":"source_concrete_Unsigned_impl_target_requires_independent_verification"},
    {"id":"sealed-signed-unsigned-abs","group":"sealed_signed","owner":"<core::primitive::{signed} as itoa::private::Sealed>::write","target":"core::primitive::{signed}::unsigned_abs","target_kind":"builtin","file":"src/lib.rs","lines":[171,171],"expression":"self.unsigned_abs()","token":"unsigned_abs","dispatch":"primitive_method_concrete_receiver","target_status":"builtin_primitive_target_unverified"},
    {"id":"sealed-signed-try-into","group":"sealed_signed","owner":"<core::primitive::{signed} as itoa::private::Sealed>::write","target":"core::convert::TryInto::try_into","target_kind":"builtin","file":"src/lib.rs","lines":[172,172],"expression":"(&mut buf[offset..]).try_into()","token":"try_into","dispatch":"trait_method_inferred_target","target_status":"external_trait_impl_target_requires_independent_verification"},
    {"id":"sealed-signed-unwrap","group":"sealed_signed","owner":"<core::primitive::{signed} as itoa::private::Sealed>::write","target":"core::result::Result::unwrap","target_kind":"builtin","file":"src/lib.rs","lines":[172,172],"expression":".unwrap()","token":"unwrap","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"sealed-signed-minus-write","group":"sealed_signed","owner":"<core::primitive::{signed} as itoa::private::Sealed>::write","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[176,176],"expression":"buf[offset].write(b'-')","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"sealed-signed-slice","group":"sealed_signed","owner":"<core::primitive::{signed} as itoa::private::Sealed>::write","target":"itoa::slice_buffer_to_str","target_kind":"local","file":"src/lib.rs","lines":[179,179],"expression":"slice_buffer_to_str(buf, offset)","token":"slice_buffer_to_str","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"pointer-sealed-delegate","group":"pointer","owner":"<core::primitive::{receiver} as itoa::private::Sealed>::write","target":"<core::primitive::{primitive} as itoa::private::Sealed>::write","target_kind":"local","file":"src/lib.rs","lines":[205,205],"expression":"(self as $primitive).write(buf)","token":"write","dispatch":"concrete_trait_impl_expectation","target_status":"pointer_Sealed_target_requires_independent_verification"},
    {"id":"unsigned-fmt-len","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::slice::<T>::len","target_kind":"builtin","file":"src/lib.rs","lines":[265,265],"expression":"buf.len()","token":"len","dispatch":"builtin_method_autoderef","target_status":"core_array_or_slice_method_target_unverified"},
    {"id":"unsigned-fmt-size-of","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::size_of","target_kind":"builtin","file":"src/lib.rs","lines":[271,271],"expression":"mem::size_of::<Self>()","token":"size_of","dispatch":"builtin_associated_function","target_status":"builtin_key_target_unverified"},
    {"id":"unsigned-fmt-try-999","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::convert::TryInto::try_into","target_kind":"builtin","file":"src/lib.rs","lines":[274,274],"expression":"999.try_into()","token":"try_into","dispatch":"trait_method_inferred_target","target_status":"external_trait_impl_target_unverified"},
    {"id":"unsigned-fmt-expect-999","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::result::Result::expect","target_kind":"builtin","file":"src/lib.rs","lines":[275,275],"expression":".expect(\"branch is not hit for types that cannot fit 999 (u8)\")","token":"expect","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-try-scale","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::convert::TryInto::try_into","target_kind":"builtin","file":"src/lib.rs","lines":[281,281],"expression":"1_00_00.try_into()","token":"try_into","dispatch":"trait_method_inferred_target","target_status":"external_trait_impl_target_unverified"},
    {"id":"unsigned-fmt-expect-scale","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::result::Result::expect","target_kind":"builtin","file":"src/lib.rs","lines":[282,282],"expression":".expect(\"branch is not hit for types that cannot fit 1E4 (u8)\")","token":"expect","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-divmod-quad","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"itoa::divmod100","target_kind":"local","file":"src/lib.rs","lines":[285,285],"expression":"divmod100(quad as u32)","token":"divmod100","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"unsigned-fmt-get-p1-low","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[288,288],"expression":".get_unchecked(pair1 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"unsigned-fmt-write-p1-low","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[287,288],"expression":"buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-get-p1-high","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[290,290],"expression":".get_unchecked(pair1 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"unsigned-fmt-write-p1-high","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[289,290],"expression":"buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-get-p2-low","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[292,292],"expression":".get_unchecked(pair2 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"unsigned-fmt-write-p2-low","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[291,292],"expression":"buf[offset + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-get-p2-high","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[294,294],"expression":".get_unchecked(pair2 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"unsigned-fmt-write-p2-high","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[293,294],"expression":"buf[offset + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-divmod-pair","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"itoa::divmod100","target_kind":"local","file":"src/lib.rs","lines":[302,302],"expression":"divmod100(remain as u32)","token":"divmod100","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"unsigned-fmt-get-low","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[306,306],"expression":".get_unchecked(pair as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"unsigned-fmt-write-low","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[305,306],"expression":"buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-get-high","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[308,308],"expression":".get_unchecked(pair as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"unsigned-fmt-write-high","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[307,308],"expression":"buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"unsigned-fmt-write-last","group":"unsigned_macro","owner":"<core::primitive::{receiver} as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[319,319],"expression":"buf[offset].write(b'0' + last)","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-len","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::slice::<T>::len","target_kind":"builtin","file":"src/lib.rs","lines":[340,340],"expression":"buf.len()","token":"len","dispatch":"builtin_method_autoderef","target_status":"core_array_or_slice_method_target_unverified"},
    {"id":"u128-fmt-write-zero","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[341,341],"expression":"buf[offset].write(b'0')","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-div-rem-first","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"itoa::div_rem_1e16","target_kind":"local","file":"src/lib.rs","lines":[345,345],"expression":"div_rem_1e16(self)","token":"div_rem_1e16","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"u128-fmt-encode-first","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"itoa::enc_16lsd","target_kind":"local","file":"src/lib.rs","lines":[350,350],"expression":"enc_16lsd::<{ u128::MAX_STR_LEN - 16 }>(buf, mod_1e16)","token":"enc_16lsd","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"u128-fmt-div-rem-second","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"itoa::div_rem_1e16","target_kind":"local","file":"src/lib.rs","lines":[353,353],"expression":"div_rem_1e16(quot_1e16)","token":"div_rem_1e16","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"u128-fmt-encode-second","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"itoa::enc_16lsd","target_kind":"local","file":"src/lib.rs","lines":[358,358],"expression":"enc_16lsd::<{ u128::MAX_STR_LEN - 32 }>(buf, mod2)","token":"enc_16lsd","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"u128-fmt-divmod-quad","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"itoa::divmod100","target_kind":"local","file":"src/lib.rs","lines":[371,371],"expression":"divmod100(quad as u32)","token":"divmod100","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"u128-fmt-get-q1","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[373,373],"expression":".get_unchecked(pair1 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"u128-fmt-write-q1","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[373,373],"expression":"buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-get-q2","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[374,374],"expression":".get_unchecked(pair1 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"u128-fmt-write-q2","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[374,374],"expression":"buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-get-q3","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[375,375],"expression":".get_unchecked(pair2 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"u128-fmt-write-q3","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[375,375],"expression":"buf[offset + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-get-q4","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[376,376],"expression":".get_unchecked(pair2 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"u128-fmt-write-q4","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[376,376],"expression":"buf[offset + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-divmod-pair","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"itoa::divmod100","target_kind":"local","file":"src/lib.rs","lines":[384,384],"expression":"divmod100(remain as u32)","token":"divmod100","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"u128-fmt-get-pair-low","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[387,387],"expression":".get_unchecked(pair as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"u128-fmt-write-pair-low","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[387,387],"expression":"buf[offset + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-get-pair-high","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[388,388],"expression":".get_unchecked(pair as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"u128-fmt-write-pair-high","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[388,388],"expression":"buf[offset + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"u128-fmt-write-last","group":"single","owner":"<core::primitive::u128 as itoa::Unsigned>::fmt","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[399,399],"expression":"buf[offset].write(b'0' + last)","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"slice-helper-get-unchecked","group":"single","owner":"itoa::slice_buffer_to_str","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[249,249],"expression":"buf.get_unchecked(offset..)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"slice-helper-from-utf8-unchecked","group":"single","owner":"itoa::slice_buffer_to_str","target":"core::str::from_utf8_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[252,252],"expression":"str::from_utf8_unchecked(&*(written as *const [MaybeUninit<u8>] as *const [u8]))","token":"from_utf8_unchecked","dispatch":"builtin_unsafe_function","target_status":"builtin_key_target_unverified"},
    {"id":"enc-rev","group":"single","owner":"itoa::enc_16lsd","target":"core::iter::DoubleEndedIterator::rev","target_kind":"builtin","file":"src/lib.rs","lines":[413,413],"expression":"(1..4).rev()","token":"rev","dispatch":"trait_method_on_range","target_status":"concrete_core_trait_method_target_unverified"},
    {"id":"enc-divmod-loop","group":"single","owner":"itoa::enc_16lsd","target":"itoa::divmod100","target_kind":"local","file":"src/lib.rs","lines":[417,417],"expression":"divmod100(quad as u32)","token":"divmod100","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"enc-divmod-tail","group":"single","owner":"itoa::enc_16lsd","target":"itoa::divmod100","target_kind":"local","file":"src/lib.rs","lines":[431,431],"expression":"divmod100(remain as u32)","token":"divmod100","dispatch":"ordinary_local_call","target_status":"local_target_occurrence_key_pending"},
    {"id":"enc-get-p1-low","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[420,420],"expression":".get_unchecked(pair1 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-p1-low","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[419,420],"expression":"buf[quad_index * 4 + OFFSET + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"enc-get-p1-high","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[422,422],"expression":".get_unchecked(pair1 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-p1-high","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[421,422],"expression":"buf[quad_index * 4 + OFFSET + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"enc-get-p2-low","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[424,424],"expression":".get_unchecked(pair2 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-p2-low","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[423,424],"expression":"buf[quad_index * 4 + OFFSET + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"enc-get-p2-high","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[426,426],"expression":".get_unchecked(pair2 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-p2-high","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[425,426],"expression":"buf[quad_index * 4 + OFFSET + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"enc-get-tail-p1-low","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[433,433],"expression":".get_unchecked(pair1 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-tail-p1-low","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[433,433],"expression":"buf[OFFSET + 0].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"enc-get-tail-p1-high","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[434,434],"expression":".get_unchecked(pair1 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-tail-p1-high","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[434,434],"expression":"buf[OFFSET + 1].write(*DECIMAL_PAIRS.0.get_unchecked(pair1 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"enc-get-tail-p2-low","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[435,435],"expression":".get_unchecked(pair2 as usize * 2 + 0)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-tail-p2-low","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[435,435],"expression":"buf[OFFSET + 2].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 0))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"enc-get-tail-p2-high","group":"single","owner":"itoa::enc_16lsd","target":"core::slice::<T>::get_unchecked","target_kind":"builtin","file":"src/lib.rs","lines":[436,436],"expression":".get_unchecked(pair2 as usize * 2 + 1)","token":"get_unchecked","dispatch":"builtin_method_autoderef","target_status":"core_slice_method_target_unverified"},
    {"id":"enc-write-tail-p2-high","group":"single","owner":"itoa::enc_16lsd","target":"core::mem::MaybeUninit::write","target_kind":"builtin","file":"src/lib.rs","lines":[436,436],"expression":"buf[OFFSET + 3].write(*DECIMAL_PAIRS.0.get_unchecked(pair2 as usize * 2 + 1))","token":"write","dispatch":"builtin_method","target_status":"builtin_method_target_unverified"},
    {"id":"div-rem-mulhi","group":"single","owner":"itoa::div_rem_1e16","target":"itoa::u128_ext::mulhi","target_kind":"local","file":"src/lib.rs","lines":[463,463],"expression":"u128_ext::mulhi(n, M_HIGH)","token":"mulhi","dispatch":"ordinary_local_call","target_status":"selected_local_target_occurrence_key_pending"},
    {"id":"mulhi-from-xlo-a","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[14,14],"expression":"u128::from(x_lo)","token":"u128::from(x_lo)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":1},
    {"id":"mulhi-from-ylo-a","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[14,14],"expression":"u128::from(y_lo)","token":"u128::from(y_lo)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":2},
    {"id":"mulhi-from-xlo-b","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[15,15],"expression":"u128::from(x_lo)","token":"u128::from(x_lo)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":1},
    {"id":"mulhi-from-yhi-a","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[15,15],"expression":"u128::from(y_hi)","token":"u128::from(y_hi)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":2},
    {"id":"mulhi-from-xhi-a","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[19,19],"expression":"u128::from(x_hi)","token":"u128::from(x_hi)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":1},
    {"id":"mulhi-from-ylo-b","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[19,19],"expression":"u128::from(y_lo)","token":"u128::from(y_lo)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":2},
    {"id":"mulhi-from-mlo","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[19,19],"expression":"u128::from(m_lo)","token":"u128::from(m_lo)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":3},
    {"id":"mulhi-from-xhi-b","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[21,21],"expression":"u128::from(x_hi)","token":"u128::from(x_hi)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":1},
    {"id":"mulhi-from-yhi-b","group":"single","owner":"itoa::u128_ext::mulhi","target":"core::convert::From<u64>::from for u128","target_kind":"builtin","file":"src/u128_ext.rs","lines":[21,21],"expression":"u128::from(y_hi)","token":"u128::from(y_hi)","dispatch":"concrete_trait_impl_expectation","target_status":"u128_From_impl_target_unverified","ordinal_on_line":2},
]


def source_bytes(source_root: Path) -> dict[str, bytes]:
    return {path: (source_root / path).read_bytes() for path in SOURCE_HASHES}


def verify_source_hashes(sources: dict[str, bytes]) -> None:
    for path, expected in SOURCE_HASHES.items():
        actual = hashlib.sha256(sources[path]).hexdigest()
        if actual != expected:
            raise ValueError(f"pinned source hash mismatch for {path}: {actual}")


def build_call_site_inventory(source_root: Path, reference_path: Path) -> None:
    sources = source_bytes(source_root)
    verify_source_hashes(sources)
    document = json.loads(reference_path.read_text())
    raw_reference = reference_path.with_name("itoa-hld-unreviewed-reference.json")
    raw_digest = hashlib.sha256(raw_reference.read_bytes()).hexdigest()
    expected_raw_digest = document["source_review_provenance"]["preserved_unreviewed_json_sha256"]
    if raw_digest != expected_raw_digest:
        raise ValueError("preserved raw draft digest changed")
    local_nodes = {
        row["label"]: row
        for row in document["selected_node_occurrences"] + document["rejected_node_candidates"]
    }
    evidence = list(document["evidence_ranges"])
    evidence_by_id = {row["id"]: row for row in evidence}
    evidence_by_span = {(row["file"], tuple(row["bytes"])): row["id"] for row in evidence}
    line_offsets = {}
    for path, data in sources.items():
        offsets = [0]
        for line in data.splitlines(keepends=True):
            offsets.append(offsets[-1] + len(line))
        line_offsets[path] = offsets

    def add_evidence(path: str, lines: list[int], label: str) -> dict:
        first, last = lines
        start, end = line_offsets[path][first - 1], line_offsets[path][last]
        existing = evidence_by_span.get((path, (start, end)))
        if existing:
            return evidence_by_id[existing]
        entry_id = f"call_site_range_{len(evidence) + 1:04d}"
        while entry_id in evidence_by_id:
            entry_id = f"call_site_range_{len(evidence) + 2:04d}"
        excerpt = sources[path][start:end]
        entry = {
            "id": entry_id,
            "file": path,
            "lines": lines,
            "bytes": [start, end],
            "sha256": hashlib.sha256(excerpt).hexdigest(),
            "evidence_label": label,
        }
        evidence.append(entry)
        evidence_by_id[entry_id] = entry
        evidence_by_span[(path, (start, end))] = entry_id
        return entry

    def invocation_evidence(evidence_id: str) -> dict:
        entry = evidence_by_id[evidence_id]
        start, end = entry["bytes"]
        raw = sources[entry["file"]][start:end]
        if hashlib.sha256(raw).hexdigest() != entry["sha256"]:
            raise ValueError("macro invocation evidence hash mismatch: " + evidence_id)
        return {**entry, "excerpt": raw.decode("utf-8")}

    def target_ref(kind: str, label: str) -> dict:
        if kind == "builtin":
            return {
                "key_type": "NameSymbolTargetKey::Builtin",
                "name": label,
                "definition_span": None,
                "identity_status": "symbolic_builtin_without_fabricated_sysroot_span",
            }
        if kind != "local" or label not in local_nodes:
            raise ValueError("call target endpoint is undefined: " + label)
        return {
            "key_type": "SemanticOccurrenceKeyValue",
            "display_label_only": label,
            "semantic_occurrence_key": None,
            "identity_status": "canonical_compilation_and_expansion_proof_pending",
            "selection": local_nodes[label].get("selection_status", local_nodes[label].get("kind")),
        }

    def expansions(group: str) -> list[tuple[dict, tuple[str, str, str, str] | None, str]]:
        if group == "single":
            return [({}, None, "source occurrence")]
        if group in ("integer_signed", "integer_unsigned", "sealed_signed", "sealed_unsigned"):
            result = []
            for signed, unsigned, invocation_id in INTEGER_PAIRS:
                invocation = f"impl_Integer!({signed}, {unsigned})"
                signed_group = group in ("integer_signed", "sealed_signed")
                for receiver in ((signed,) if signed_group else (unsigned,)):
                    values = {"signed": signed, "unsigned": unsigned, "receiver": receiver}
                    macro = ("impl_Integer", invocation, invocation_id, f"{signed}/{unsigned}; receiver={receiver}")
                    result.append((values, macro, f"{signed}/{unsigned}; receiver={receiver}"))
            return result
        if group == "pointer":
            return [
                ({"receiver": receiver, "primitive": primitive},
                 ("impl_Integer_size", f"impl_Integer_size!({receiver} as {primitive} #[cfg(target_pointer_width = \"64\")])", invocation_id, f"{receiver} as {primitive}"),
                 f"{receiver} as {primitive}")
                for receiver, primitive, invocation_id in POINTER_PAIRS
            ]
        if group == "unsigned_macro":
            return [
                ({"receiver": receiver}, ("impl_Unsigned", f"impl_Unsigned!({receiver})", invocation_id, receiver), receiver)
                for receiver, invocation_id in UNSIGNED_RECEIVERS
            ]
        raise ValueError("unknown macro expansion group: " + group)

    def build_row(template: dict, values: dict, macro: tuple | None, substitution: str) -> dict:
        owner = template["owner"].format_map(values)
        target = template["target"].format_map(values)
        if owner not in local_nodes:
            raise ValueError("body owner endpoint is undefined: " + owner)
        source_range = add_evidence(template["file"], template["lines"], "call template " + template["id"])
        source = sources[template["file"]]
        start, end = source_range["bytes"]
        excerpt = source[start:end]
        token = template["token"].encode("utf-8")
        token_offset = excerpt.find(token)
        if token_offset < 0:
            raise ValueError(f"callee token {template['token']!r} not found for {template['id']}")
        token_range = [start + token_offset, start + token_offset + len(token)]
        expression_bytes = template["expression"].encode("utf-8")
        expression_offset = excerpt.find(expression_bytes)
        expression_range = None if expression_offset < 0 else [start + expression_offset, start + expression_offset + len(expression_bytes)]
        macro_record = None
        evidence_ids = [source_range["id"]]
        if macro:
            invocation = invocation_evidence(macro[2])
            evidence_ids.append(macro[2])
            macro_record = {
                "template_macro": macro[0],
                "invocation_identity": macro[1],
                "invocation_evidence": invocation,
                "receiver_substitution": macro[3],
                "expanded_byte_range": None,
                "expanded_semantic_occurrence_key": None,
            }
        return {
            "site_occurrence_key_draft": template["id"] + "@" + substitution,
            "lexical_template_key": template["id"],
            "relation_kind": "calls",
            "body_owner": {
                "display_label": owner,
                "owner_site_kind": template.get("owner_kind", "function_or_associated_item_body"),
                "semantic_occurrence_key": None,
                "identity_status": "source label only; canonical occurrence proof pending",
                "selection": local_nodes[owner].get("selection_status", local_nodes[owner].get("kind")),
            },
            "target_candidates": [{
                "reference": target_ref(template["target_kind"], target),
                "target_status": template["target_status"],
                "dispatch_kind": template["dispatch"],
                "concrete_fanout": "none" if template["dispatch"] == "generic_trait_declaration" else None,
            }],
            "source_site": {
                "file": template["file"],
                "line_range_inclusive": template["lines"],
                "byte_range_half_open": source_range["bytes"],
                "source_excerpt_sha256": source_range["sha256"],
                "source_evidence_range_id": source_range["id"],
                "source_excerpt": excerpt.decode("utf-8"),
                "call_expression_template": template["expression"],
                "call_expression_byte_range": expression_range,
                "callee_token": template["token"],
                "callee_token_byte_range": token_range,
                "ordinal_on_source_line": template.get("ordinal_on_line"),
            },
            "macro_origin": macro_record,
            "source_evidence_range_ids": evidence_ids,
            "fact_status": "draft_source_derived_expectation_not_extracted",
        }

    rows = []
    for template in CALL_TEMPLATES:
        for values, macro, substitution in expansions(template["group"]):
            rows.append(build_row(template, values, macro, substitution))

    non_call_macro_sites = []
    for signed, unsigned, invocation_id in INTEGER_PAIRS:
        invocation = invocation_evidence(invocation_id)
        for line, macro_text, substitution in (
            (138, "assert!($Signed::MIN < 0, \"need signed\")", f"Signed={signed}"),
            (139, "assert!($Unsigned::MIN == 0, \"need unsigned\")", f"Unsigned={unsigned}"),
            (140, "assert!($Signed::BITS == $Unsigned::BITS, \"need counterparts\")", f"Signed={signed}; Unsigned={unsigned}"),
        ):
            source_range = add_evidence("src/lib.rs", [line, line], "compile-time assert macro template")
            non_call_macro_sites.append({
                "macro_site_key_draft": f"impl_Integer-assert-line-{line}@{signed}/{unsigned}",
                "macro_name": "assert!",
                "source_template_evidence": source_range,
                "macro_template_text": macro_text,
                "invocation_identity": f"impl_Integer!({signed}, {unsigned})",
                "invocation_evidence": invocation,
                "receiver_substitution": substitution,
                "source_owner_kind": "anonymous_const_initializer_inside_impl_Integer_macro",
                "relation_kind": None,
                "classification": "active_macro_invocation_not_an_explicit_call_or_method_site",
            })
    debug_assert_evidence = add_evidence("src/lib.rs", [232, 232], "active debug_assert macro invocation")
    non_call_macro_sites.append({
        "macro_site_key_draft": "divmod100-debug-assert",
        "macro_name": "debug_assert!",
        "source_template_evidence": debug_assert_evidence,
        "macro_template_text": "debug_assert!(value < 10_000)",
        "invocation_identity": None,
        "invocation_evidence": None,
        "receiver_substitution": None,
        "source_owner_kind": "function_body",
        "source_owner_label": "itoa::divmod100",
        "relation_kind": None,
        "classification": "active_with_dev_debug_assertions_but_not_a_function_or_method_call_site",
    })
    for site in non_call_macro_sites:
        evidence_row = site["source_template_evidence"]
        raw = sources[evidence_row["file"]][evidence_row["bytes"][0]:evidence_row["bytes"][1]]
        if hashlib.sha256(raw).hexdigest() != evidence_row["sha256"]:
            raise ValueError("non-call macro source evidence hash mismatch")
        owner_label = site.get("source_owner_label")
        if owner_label is not None and owner_label not in local_nodes:
            raise ValueError("non-call macro source owner endpoint is undefined")

    for row in rows:
        for evidence_id in row["source_evidence_range_ids"]:
            entry = evidence_by_id[evidence_id]
            raw = sources[entry["file"]][entry["bytes"][0]:entry["bytes"][1]]
            if hashlib.sha256(raw).hexdigest() != entry["sha256"]:
                raise ValueError("source evidence hash mismatch: " + evidence_id)
        for candidate in row["target_candidates"]:
            if candidate["reference"]["key_type"] not in ALLOWED_TARGET_KEY_TYPES:
                raise ValueError("unsupported endpoint key type")

    if len(CALL_TEMPLATES) != 93 or len(rows) != 197:
        raise ValueError(f"call-site count mismatch: templates={len(CALL_TEMPLATES)} occurrences={len(rows)}")
    row_keys = [row["site_occurrence_key_draft"] for row in rows]
    if len(row_keys) != len(set(row_keys)):
        raise ValueError("duplicate call-site occurrence key")
    bodyless = {"itoa::private::Sealed::write", "itoa::Unsigned::fmt"}
    if any(row["body_owner"]["display_label"] in bodyless for row in rows):
        raise ValueError("call body attributed to a bodyless trait declaration")
    if any(row["relation_kind"] != "calls" for row in rows):
        raise ValueError("non-call relation found in call-site-only inventory")
    macro_counts = {}
    for row in rows:
        macro = row["macro_origin"]
        if macro:
            macro_counts[macro["template_macro"]] = macro_counts.get(macro["template_macro"], 0) + 1
    expected_macro_counts = {"impl_Integer": 50, "impl_Integer_size": 2, "impl_Unsigned": 84}
    if macro_counts != expected_macro_counts:
        raise ValueError(f"macro call multiplicity mismatch: {macro_counts}")

    integer_macro = [row for row in rows if row["lexical_template_key"].startswith("max-len-")]
    pointer_calls = [row for row in rows if row["lexical_template_key"] == "pointer-sealed-delegate"]
    fmt_bodies = {
        row["label"] for row in document["rejected_node_candidates"]
        if row["kind"] == "private_trait_method_implementation"
    }
    fmt_call_owners = {
        row["body_owner"]["display_label"] for row in rows
        if row["body_owner"]["display_label"].endswith("as itoa::Unsigned>::fmt")
    }
    sealed_body_nodes = {
        row["label"] for row in document["selected_node_occurrences"]
        if row["kind"] == "trait_method_implementation"
    }
    sealed_call_owners = {
        row["body_owner"]["display_label"] for row in rows
        if "as itoa::private::Sealed>::write" in row["body_owner"]["display_label"]
    }
    if len(integer_macro) != 10 or len(pointer_calls) != 2:
        raise ValueError("MAX_STR_LEN or pointer delegation multiplicity is incomplete")
    if not fmt_bodies.issubset(fmt_call_owners) or len(fmt_bodies) != 5:
        raise ValueError("five distinct private Unsigned::fmt bodies are not call owners")
    if not sealed_body_nodes.issubset(sealed_call_owners) or len(sealed_body_nodes) != 12:
        raise ValueError("ten fixed and two pointer Sealed writer bodies are not represented")
    from_rows = [row for row in rows if row["lexical_template_key"].startswith("mulhi-from-")]
    from_counts = {}
    for row in from_rows:
        line = str(row["source_site"]["line_range_inclusive"][0])
        from_counts[line] = from_counts.get(line, 0) + 1
    if from_counts != {"14": 2, "15": 2, "19": 3, "21": 2}:
        raise ValueError(f"u128::from site count mismatch: {from_counts}")
    generic_format = next(row for row in rows if row["lexical_template_key"] == "format-generic-sealed-write")
    if generic_format["target_candidates"][0]["concrete_fanout"] != "none":
        raise ValueError("generic Buffer::format must not fan out to concrete writers")
    if "superseded_unreviewed_edge_families" not in document:
        raise ValueError("prior edge families must remain preserved and unreviewed")
    if "candidate_hld_projection" in document:
        raise ValueError("this bounded pass must not create an HLD collapse")

    if len(non_call_macro_sites) != 16:
        raise ValueError("active assert/debug_assert macro accounting mismatch")
    evidence.sort(key=lambda row: (row["file"], row["bytes"][0], row["bytes"][1], row["id"]))
    document["evidence_ranges"] = evidence
    document["counts"]["evidence_ranges"] = len(evidence)
    document["call_site_inventory"] = {
        "status": "SOURCE_DERIVED_CALL_SITES_DRAFT_UNVERIFIED",
        "generation_mode": "deterministic_offline_source_expectations_only",
        "production_mapper_invoked": False,
        "production_interface_or_data_created": False,
        "relation_kind": "calls",
        "allowed_relation_kind_used": True,
        "lexical_template_count": len(CALL_TEMPLATES),
        "active_call_occurrence_count": len(rows),
        "call_sites": rows,
        "non_call_macro_invocations": non_call_macro_sites,
        "non_call_macro_template_count": 4,
        "active_non_call_macro_invocation_count": len(non_call_macro_sites),
        "macro_occurrence_counts": macro_counts,
        "active_macro_invocations": {
            "impl_Integer": [item[0] for item in INTEGER_PAIRS],
            "impl_Integer_size": ["isize as i64", "usize as u64"],
            "impl_Unsigned": [item[0] for item in UNSIGNED_RECEIVERS],
            "manual_Unsigned": ["u128"],
            "compile_time_assert_expansions": 15,
            "debug_assert": ["divmod100; active because dev debug assertions are enabled"],
        },
        "inactive_pointer_invocations": ["isize as i16", "usize as u16", "isize as i32", "usize as u32"],
        "required_site_counts": {
            "Buffer_format_calls": 5,
            "generated_MAX_STR_LEN_ilog10": 10,
            "fixed_Sealed_write_bodies": 10,
            "pointer_Sealed_write_delegations": 2,
            "distinct_private_Unsigned_fmt_bodies": 5,
            "enc_16lsd_rev": 1,
            "mulhi_u128_from": 9,
            "mulhi_from_line_counts": from_counts,
            "compile_time_assert_invocations_not_call_edges": 15,
            "active_debug_assert_invocations_not_call_edges": 1,
        },
        "completeness_notes": [
            "Explicit call and method sites in src/lib.rs and src/u128_ext.rs were reviewed; one tuple-struct constructor in the DECIMAL_PAIRS static initializer is included as a source candidate.",
            "Fifteen compile-time assert! expansions and one active debug_assert! invocation are accounted separately as macro invocations, not fabricated calls relations.",
            "The call count is syntax-site occurrence count after active source macro substitutions, not runtime loop iteration count.",
            "Operators, indexing, for-loop iteration, autoderef, reborrow, and other implicit compiler operations are not fabricated as explicit calls.",
            "Macro-expanded byte offsets and canonical SemanticOccurrenceKeyValue identities remain unresolved; no expansion ranges or occurrence hashes are invented.",
        ],
        "unresolved_target_and_proof_items": [
            "Canonical local SemanticOccurrenceKeyValue identities and compiler-backed macro expansion/source-map provenance.",
            "Canonical core/sysroot identities for primitive methods, builtin functions, and trait methods.",
            "Concrete Unsigned::fmt and pointer Sealed::write implementation targets require independent target verification.",
            "The DECIMAL_PAIRS tuple-struct constructor target and its eventual relation classification need semantic verification.",
            "assert! and debug_assert! macro expansions are accounted as macro invocations only; no lowered panic/call target is asserted.",
            "Operator, index, loop, autoderef/reborrow, and other compiler-implicit operations are outside the explicit source call inventory.",
            "HLD selection, terminal-kind collapse, paths, and rollups are intentionally not computed in this step.",
        ],
        "validation": {
            "all_body_owners_are_source_nodes": True,
            "all_local_target_candidates_are_selected_or_rejected_source_nodes": True,
            "builtin_targets_use_NameSymbolTargetKey_Builtin": True,
            "no_bodyless_trait_declaration_owns_a_body_call": True,
            "macro_multiplicities_validated": True,
            "source_and_invocation_ranges_hashed": True,
        },
    }
    document["source_review_provenance"]["scope"] = (
        "source-only node inventory and bounded explicit call-site inventory; no production implementation or compiler verification"
    )
    document["source_review_provenance"]["call_site_inventory_workflow_failures"] = [
        {"attempt": 1, "error": "SyntaxError: Unexpected identifier 'itoa'", "stage": "JavaScript wrapper parse; apply_patch not called"},
        {"attempt": 2, "error": "SyntaxError: Missing } in template expression", "stage": "JavaScript wrapper parse; apply_patch not called"},
        {"attempt": 3, "error": "SyntaxError: Unexpected identifier 'text'", "stage": "JavaScript wrapper parse; apply_patch not called"},
    ]
    corrections = document["source_review_provenance"].setdefault("review_corrections", [])
    inventory_correction = "explicit calls inventoried by concrete body owner, source range, target candidate, and active macro substitution"
    if inventory_correction not in corrections:
        corrections.append(inventory_correction)
    parser_failure_note = (
        "Three earlier full-generator submissions failed JavaScript-wrapper parsing before apply_patch: "
        "Unexpected identifier 'itoa'; Missing } in template expression; Unexpected identifier 'text'. "
        "No files were changed by those failed submissions. The current call-site-only script is authored in bounded JSON-escaped patches."
    )
    if parser_failure_note not in corrections:
        corrections.append(parser_failure_note)
    document["counts"]["lexical_call_site_templates"] = len(CALL_TEMPLATES)
    document["counts"]["explicit_call_site_occurrences"] = len(rows)
    added_unknowns = [
        "Call target identities remain source-derived expectations: core/sysroot identities and method/trait resolution need independent compiler/oracle proof.",
        "Concrete Unsigned::fmt and pointer Sealed::write targets are expectations, not extracted compiler resolutions.",
        "Macro-expanded occurrence identities and expansion byte ranges remain unverified; the inventory preserves template and invocation spans only.",
        "The DECIMAL_PAIRS tuple-struct constructor target and its relation classification remain unverified.",
        "assert! and debug_assert! expansions are recorded only as macro invocations; their lowered call targets are not inferred.",
        "Implicit operator/index/loop/autoderef operations and HLD collapse are outside this call-site-only step.",
    ]
    for unknown in added_unknowns:
        if unknown not in document["unknowns"]:
            document["unknowns"].append(unknown)
    if document.get("status") != "DRAFT_NOT_FROZEN":
        raise ValueError("global draft status changed unexpectedly")

    write_call_site_artifacts(reference_path, document, rows)
    print("GENERATION=offline-source-expectations-only; PRODUCTION_MAPPER=false")
    print("SOURCE_HASHES=" + json.dumps({path: hashlib.sha256(data).hexdigest() for path, data in sources.items()}, sort_keys=True))
    print(f"LEXICAL_TEMPLATES={len(CALL_TEMPLATES)} ACTIVE_CALL_OCCURRENCES={len(rows)} NON_CALL_MACRO_INVOCATIONS={len(non_call_macro_sites)}")
    print("MACRO_CALL_COUNTS=" + json.dumps(macro_counts, sort_keys=True))
    print(f"BODY_OWNER_ENDPOINTS={len({row['body_owner']['display_label'] for row in rows})} LOCAL_TARGETS={sum(item['reference']['key_type'] == 'SemanticOccurrenceKeyValue' for row in rows for item in row['target_candidates'])}")
    print("REQUIRED_COUNTS=" + json.dumps(document["call_site_inventory"]["required_site_counts"], sort_keys=True))
    print("BODYLESS_OWNER_CALLS=0 DUPLICATE_SITE_KEYS=0 UNRESOLVED_LOCAL_ENDPOINTS=0")
    print("EVIDENCE_RANGES=" + str(len(evidence)) + " RANGE_AND_HASH_VALIDATION=PASS")
    print("RAW_DRAFT_SHA256=" + raw_digest + " PRESERVED=true")
    print("DRAFT=preserved EDGE_FAMILIES=SUPERSEDED_UNREVIEWED HLD_COLLAPSE=NOT_CREATED")
    print("CALL_SITE_VALIDATION=PASS")


def write_call_site_artifacts(reference_path: Path, document: dict, rows: list[dict]) -> None:
    reference_path.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    markdown_path = reference_path.with_suffix(".md")
    markdown = markdown_path.read_text()
    start = markdown.index("## Prior edges superseded")
    end = markdown.index("## Review provenance and preserved raw draft", start)
    kinds = {}
    for row in rows:
        kind = row["lexical_template_key"].split("-")[0]
        kinds[kind] = kinds.get(kind, 0) + 1
    lines = [
        "## Occurrence-separated explicit call-site inventory",
        "",
        "Status: SOURCE_DERIVED_CALL_SITES_DRAFT_UNVERIFIED. This is deterministic offline expected-data generation from the pinned source, not mapper output, extraction, canonical identity, or an HLD collapse.",
        "",
        f"- Lexical source/template call sites: {document['call_site_inventory']['lexical_template_count']}",
        f"- Active source call/method occurrences after the configured macro substitutions: {document['call_site_inventory']['active_call_occurrence_count']}",
        "- Relation kind: calls only. Dispatch remains metadata; no concrete fan-out is assigned to generic Buffer::format<I>.",
        "- Macro expansions retain definition/template line+byte ranges and invocation identity/substitution. No macro expansion byte offsets are claimed.",
        "- The DECIMAL_PAIRS tuple-struct constructor is recorded as a call candidate owned by a static initializer; constructor-specific semantic target identity remains unresolved.",
        "- Fifteen compile-time assert! expansions and one active debug_assert! invocation are accounted separately as macros, not as calls relations.",
        "- The original edge-family material remains in JSON as SUPERSEDED_UNREVIEWED; no signature expansion, graph collapse, paths, or rollups are included in this bounded step.",
        "",
        "### Explicit call-site rows",
        "",
    ]
    for row in rows:
        site = row["source_site"]
        target = row["target_candidates"][0]
        target_ref = target["reference"]
        target_name = target_ref.get("display_label_only", target_ref.get("name"))
        span = f"{site['file']}:{site['line_range_inclusive'][0]}-{site['line_range_inclusive'][1]} bytes={site['byte_range_half_open']} sha256={site['source_excerpt_sha256']}"
        macro = row["macro_origin"]
        macro_text = "source-written"
        if macro:
            macro_text = (
                f"{macro['invocation_identity']}; substitution={macro['receiver_substitution']}; "
                f"invocation-evidence={macro['invocation_evidence']['id']}"
            )
        lines.append(
            f"- `{row['site_occurrence_key_draft']}`: `{row['body_owner']['display_label']}` ({row['body_owner']['owner_site_kind']}) calls `{target_name}` "
            f"({target['dispatch_kind']}; {target['target_status']}); {span}; "
            f"callee-byte-range={site['callee_token_byte_range']}; template={site['source_evidence_range_id']}; {macro_text}."
        )
    lines.extend([
        "",
        "### Active macro invocations outside call relations",
        "",
    ])
    for site in document["call_site_inventory"]["non_call_macro_invocations"]:
        evidence_row = site["source_template_evidence"]
        invocation_text = site["invocation_identity"] or "source-written invocation"
        substitution_text = site["receiver_substitution"] or "no macro substitution"
        lines.append(
            f"- `{site['macro_site_key_draft']}`: `{site['macro_name']}` at {evidence_row['file']}:{evidence_row['lines']} "
            f"bytes={evidence_row['bytes']} sha256={evidence_row['sha256']}; {invocation_text}; {substitution_text}; "
            f"{site['classification']}."
        )
    lines.extend([
        "",
        "### Verified accounting and unresolved proof",
        "",
        "- Required source counts validated: 10 generated MAX_STR_LEN ilog10 sites; 10 fixed-width Sealed writers; 2 pointer Sealed delegations; 5 private Unsigned::fmt body owners; 1 enc_16lsd rev; 9 u128::from sites in mulhi (line counts 14:2, 15:2, 19:3, 21:2).",
        "- Buffer::format owns its five explicit calls: as_mut_ptr, pointer cast, generic Sealed::write declaration target, str::len, unreachable_unchecked.",
        "- Concrete Unsigned::fmt and pointer Sealed::write targets remain source-derived expectations requiring independent target verification. Core/sysroot identities are symbolic Builtin candidates without fabricated definition spans.",
        "- Unresolved target/proof list is in JSON: local occurrence keys, macro expansion provenance, core/sysroot identities, tuple-struct constructor semantics, macro-lowered assertions, and implicit compiler operations.",
        "- Operators, indexing, loop iteration, autoderef/reborrow, docs-only examples and other implicit operations are not fabricated as explicit calls. Compiler expansion identities and canonical source-map occurrence keys remain unresolved.",
        "- Global status remains DRAFT. This file is a source-review reference only; it is not a machine golden or completeness claim.",
        "",
    ])
    provenance = (
        "Call-site provenance: three earlier full-generator submissions failed in the JavaScript wrapper before tool execution "
        "(Unexpected identifier 'itoa'; Missing } in template expression; Unexpected identifier 'text'). "
        "No changes were applied by those failed submissions. This call-site-only script was then authored in bounded, JSON-escaped patch chunks."
    )
    markdown = markdown[:start] + "\n".join(lines) + markdown[end:]
    marker = "## Review provenance and preserved raw draft\n"
    if provenance not in markdown:
        markdown = markdown.replace(marker, marker + "\n" + provenance + "\n", 1)
    markdown_path.write_text(markdown)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--reference", type=Path, required=True)
    args = parser.parse_args()
    build_call_site_inventory(args.source_root, args.reference)


if __name__ == "__main__":
    main()
