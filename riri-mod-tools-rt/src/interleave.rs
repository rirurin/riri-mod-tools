use std::{
    error::Error,
    fmt::Display,
    mem::MaybeUninit
};
use riri_mod_tools_proc::interleave_auto;

pub fn encode_delta_dif(data: &mut [u8]) {
    let mut prev: u8 = 0;
    for byte in data.iter_mut() {
        let v = *byte;
        *byte = v.wrapping_sub(prev);
        prev = v;
    }
}

pub fn decode_delta_dif(data: &mut [u8]) {
    let mut prev: u8 = 0;
    for byte in data.iter_mut() {
        let v = *byte;
        let decoded = prev.wrapping_add(v);
        *byte = decoded;
        prev = decoded;
    }
}

pub const INTERLEAVE_SLICE_LENGTH: usize = size_of::<u32>();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterleaveErrorReason {
    StreamTooShort,
    TrailingData,
    CorruptData,
    StreamTooLarge
}

#[derive(Debug)]
pub struct InterleaveError {
    expected_size: usize,
    actual_size: usize,
    reason: InterleaveErrorReason
}
impl InterleaveError {
    pub fn new(expected_size: usize, actual_size: usize, 
        reason: InterleaveErrorReason) -> Self {
        Self { expected_size, actual_size, reason }
    }
}
impl Error for InterleaveError {}
impl Display for InterleaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InterleaveError: {:?} (expected size: {}, actual size: {})",
            self.reason, self.expected_size, self.actual_size)
    }
}

pub trait Interleave
where Self: Sized + Copy
{
    // Array (array count is known at compile time)
    unsafe fn interleave_array<const N: usize>(data: &[Self; N]) -> [Self; N] {
        let mut buf = MaybeUninit::uninit();
        let slice = unsafe { std::slice::from_raw_parts_mut(
            buf.as_mut_ptr() as *mut u8, N * size_of::<Self>()) };
        Self::interleave_array_inner(slice, data);
        encode_delta_dif(std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, size_of::<Self>() * N));
        buf.assume_init()
    }
    unsafe fn interleave_array_inner<const N: usize>(outer: &mut [u8], data: &[Self; N]);
    fn fmt_interleaved_array<const N: usize>(arr: &[Self; N]) -> String
    where Self: std::fmt::Debug
    {
        let as_bytes = unsafe { std::slice::from_raw_parts(
            arr.as_ptr() as *const u8, N * size_of::<Self>()) };
        format!("{:?}", as_bytes)
    }
    unsafe fn deinterleave_array<const N: usize>(data: &mut [u8]) -> Result<[Self; N], InterleaveError> {
        let expected_size = N * size_of::<Self>();
        let actual_size = data.len();
        let error_code = {
            if expected_size > actual_size { Some(InterleaveErrorReason::StreamTooShort)}
            else if actual_size % expected_size != 0 { Some(InterleaveErrorReason::TrailingData )}
            else { None }
        };
        if let Some(e) = error_code {
            return Err(InterleaveError::new(expected_size, actual_size, e));
        }
        let mut buf = MaybeUninit::uninit();
        let slice = unsafe { std::slice::from_raw_parts_mut(
            buf.as_mut_ptr() as *mut u8, N * size_of::<Self>()) };
        decode_delta_dif(data);
        Self::deinterleave_array_inner::<N>(slice, data)?;
        Ok(buf.assume_init())
    }
    unsafe fn deinterleave_array_inner<const N: usize>(outer: &mut [u8], data: &[u8]) -> Result<(), InterleaveError>;
    // Slice (array count not known at compile time)
    unsafe fn interleave_slice(data: &[Self]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>() * data.len() + INTERLEAVE_SLICE_LENGTH);
        Self::interleave_slice_borrowed(&mut buf, data);
        buf
    }
    unsafe fn interleave_slice_borrowed(buf: &mut Vec<u8>, data: &[Self]) {
        buf.set_len(INTERLEAVE_SLICE_LENGTH);
        Self::interleave_slice_inner(buf, data);
        encode_delta_dif(std::slice::from_raw_parts_mut(
            buf.as_mut_ptr().add(INTERLEAVE_SLICE_LENGTH) as *mut u8, size_of::<Self>() * data.len()));
        let len_serial = (data.len() as u32).to_le_bytes();
        std::ptr::copy_nonoverlapping(
            len_serial.as_ptr(), 
            buf.as_mut_ptr(), 
            INTERLEAVE_SLICE_LENGTH);
    }
    unsafe fn interleave_slice_without_len(data: &[Self]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>() * data.len());
        Self::interleave_slice_inner(&mut buf, data);
        encode_delta_dif(std::slice::from_raw_parts_mut(
            buf.as_mut_ptr() as *mut u8, size_of::<Self>() * data.len()));
        buf
    }
    unsafe fn interleave_slice_inner(outer: &mut Vec<u8>, data: &[Self]);
    unsafe fn deinterleave_slice(data: &mut [u8]) -> Result<Vec<Self>, InterleaveError> {
        let size = u32::from_le_bytes(data[0..INTERLEAVE_SLICE_LENGTH].try_into().unwrap());
        let len = size_of::<Self>() * size as usize;
        let expected_size = len;
        let actual_size = data.len() - INTERLEAVE_SLICE_LENGTH;
        let error_code = {
            if expected_size > actual_size { Some(InterleaveErrorReason::StreamTooShort)}
            else if actual_size % expected_size != 0 { Some(InterleaveErrorReason::TrailingData )}
            else { None }
        };
        if let Some(e) = error_code {
            return Err(InterleaveError::new(expected_size, actual_size, e));
        }
        let mut buf: Vec<Self> = Vec::with_capacity(size as usize);
        unsafe { buf.set_len(size as usize) };
        let slice = std::slice::from_raw_parts_mut(data.as_mut_ptr().add(INTERLEAVE_SLICE_LENGTH), len);
        decode_delta_dif(slice);
        let buf_slice = std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, len);
        Self::deinterleave_slice_inner(buf_slice, slice)?;
        Ok(buf)
    }
    unsafe fn deinterleave_slice_without_len(data: &mut [u8], size: u32) -> Result<Vec<Self>, InterleaveError> {
        let len = size_of::<Self>() * size as usize;
        let expected_size = len;
        let actual_size = data.len();
        let error_code = {
            if expected_size > actual_size { Some(InterleaveErrorReason::StreamTooShort)}
            else if actual_size % expected_size != 0 { Some(InterleaveErrorReason::TrailingData )}
            else { None }
        };
        if let Some(e) = error_code {
            return Err(InterleaveError::new(expected_size, actual_size, e));
        }
        let mut buf: Vec<Self> = Vec::with_capacity(size as usize);
        unsafe { buf.set_len(size as usize) };
        let slice = std::slice::from_raw_parts_mut(data.as_mut_ptr(), len);
        decode_delta_dif(slice);
        let buf_slice = std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, len);
        Self::deinterleave_slice_inner(buf_slice, slice)?;
        Ok(buf)
    }
    unsafe fn deinterleave_slice_inner(outer: &mut [u8], data: &[u8]) -> Result<(), InterleaveError>;
}

impl<T, const M: usize> Interleave for [T; M]
where T: Interleave + std::fmt::Debug
{
    unsafe fn interleave_array_inner<const N: usize>(outer: &mut [u8], data: &[Self; N]) {
        for i in 0..M {
            let field = std::array::from_fn::<_, N, _>(|k| data[k][i]);
            let start = i * N * size_of::<T>();
            let end = (i + 1) * N * size_of::<T>();
            T::interleave_array_inner(&mut outer[start..end], &field);
        }
    }
    unsafe fn deinterleave_array_inner<const N: usize>(outer: &mut [u8], data: &[u8]) -> Result<(), InterleaveError> {
        for i in 0..M {
            let mut field: MaybeUninit<[T; N]> = MaybeUninit::uninit();
            let field_slice = std::slice::from_raw_parts_mut(
                field.as_mut_ptr() as *mut u8, N * size_of::<T>());
            let start = i * N * size_of::<T>();
            let end = (i + 1) * N * size_of::<T>();
            T::deinterleave_array_inner::<N>(field_slice, &data[start..end])?;
            for j in 0..N {
                std::ptr::copy_nonoverlapping(
                    field_slice.as_ptr().add(j * size_of::<T>()),
                    outer.as_mut_ptr().add((j * M + i) * size_of::<T>()),
                    size_of::<T>());
            }
        }
        Ok(())
    }
    unsafe fn interleave_slice_inner(outer: &mut Vec<u8>, data: &[Self]) {
        let len = data.len();
        let mut field = Vec::with_capacity(len);
        field.set_len(len);
        for i in 0..M {
            for k in 0..data.len() { field[k] = data[k][i] }
            T::interleave_slice_inner(outer, field.as_slice());
        }
    }
    unsafe fn deinterleave_slice_inner(outer: &mut [u8], data: &[u8]) -> Result<(), InterleaveError> {
        let field_len = data.len() / M;
        let mut field = Vec::with_capacity(field_len);
        field.set_len(field_len);
        for i in 0..M {
            let field_slice = std::slice::from_raw_parts_mut(
                field.as_mut_ptr() as *mut u8, data.len() * size_of::<T>());
            let start = i * field_len;
            let end = (i + 1) * field_len;
            T::deinterleave_slice_inner(field.as_mut_slice(), &data[start..end])?;
            for j in 0..field_len {
                std::ptr::copy_nonoverlapping(
                    field_slice.as_ptr().add(j * size_of::<T>()),
                    outer.as_mut_ptr().add((j * M + i) * size_of::<T>()),
                    size_of::<T>());
            }
        }
        Ok(())
    }
}

interleave_auto!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
);
