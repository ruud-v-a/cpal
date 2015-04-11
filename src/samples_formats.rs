use std::borrow::Cow;
use std::mem;

/// Format that each sample has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleFormat {
    /// The value 0 corresponds to 0.
    I16,
    /// The value 0 corresponds to 32768.
    U16,
    /// The value 0 corresponds to 8388608.
    U24,
    /// The boundaries are (-1.0, 1.0).
    F32,
}

impl SampleFormat {
    /// Returns the size in bytes of a sample of this format.
    pub fn get_sample_size(&self) -> usize {
        match self {
            &SampleFormat::I16 => mem::size_of::<i16>(),
            &SampleFormat::U16 => mem::size_of::<u16>(),
            &SampleFormat::U24 => mem::size_of::<u32>(),
            &SampleFormat::F32 => mem::size_of::<f32>(),
        }
    }
}

/// Trait for containers that contain PCM data.
#[unstable = "Will be rewritten with associated types"]
pub trait Sample: Copy + Clone {
    fn get_format(Option<Self>) -> SampleFormat;

    /// Returns `(self + other) / 2`.
    fn interpolate(self, other: Self) -> Self;

    /// Turns the data into samples of type `I16`.
    fn to_vec_i16(&[Self]) -> Cow<[i16]>;
    /// Turns the data into samples of type `U16`.
    fn to_vec_u16(&[Self]) -> Cow<[u16]>;
    /// Turns the data into samples of type `U24`.
    fn to_vec_u24(&[Self]) -> Cow<[u32]>;
    /// Turns the data into samples of type `F32`.
    fn to_vec_f32(&[Self]) -> Cow<[f32]>;
}

impl Sample for u16 {
    fn get_format(_: Option<u16>) -> SampleFormat {
        SampleFormat::U16
    }

    fn interpolate(self, other: u16) -> u16 {
        (self + other) / 2
    }

    fn to_vec_i16(input: &[u16]) -> Cow<[i16]> {
        Cow::Owned(input.iter().map(|&value| {
            if value >= 32768 {
                (value - 32768) as i16
            } else {
                (value as i16) - 32767 - 1
            }
        }).collect())
    }

    fn to_vec_u16(input: &[u16]) -> Cow<[u16]> {
        Cow::Borrowed(input)
    }

    fn to_vec_u24(input: &[u16]) -> Cow<[u32]> {
        Cow::Owned(input.iter().map(|&value| {
            // Widen 16 to 24 bits.
            (value as u32) << 8
        }).collect())
    }

    fn to_vec_f32(input: &[u16]) -> Cow<[f32]> {
        Cow::Owned(Sample::to_vec_f32(&Sample::to_vec_i16(input)).to_vec())
    }
}

impl Sample for i16 {
    fn get_format(_: Option<i16>) -> SampleFormat {
        SampleFormat::I16
    }

    fn interpolate(self, other: i16) -> i16 {
        (self + other) / 2
    }

    fn to_vec_i16(input: &[i16]) -> Cow<[i16]> {
        Cow::Borrowed(input)
    }

    fn to_vec_u16(input: &[i16]) -> Cow<[u16]> {
        Cow::Owned(input.iter().map(|&value| {
            if value < 0 {
                (value + 32767 + 1) as u16
            } else {
                (value as u16) + 32768
            }
        }).collect())
    }

    fn to_vec_u24(input: &[i16]) -> Cow<[u32]> {
        Cow::Owned(Sample::to_vec_u24(&Sample::to_vec_u16(input)).to_vec())
    }

    fn to_vec_f32(input: &[i16]) -> Cow<[f32]> {
        Cow::Owned(input.iter().map(|&value| {
            if value > 0 {
                value as f32 / 32767.0
            } else {
                value as f32 / 32768.0
            }
        }).collect())
    }
}

impl Sample for u32 {
    fn get_format(_: Option<u32>) -> SampleFormat {
        SampleFormat::U24
    }

    fn interpolate(self, other: u32) -> u32 {
        (self + other) / 2
    }

    fn to_vec_i16(input: &[u32]) -> Cow<[i16]> {
        Cow::Owned(Sample::to_vec_i16(&Sample::to_vec_u16(input)).to_vec())
    }

    fn to_vec_u16(input: &[u32]) -> Cow<[u16]> {
        Cow::Owned(input.iter().map(|&value| {
            // Truncate to 16 bits.
            (value >> 8) as u16
        }).collect())
    }

    fn to_vec_u24(input: &[u32]) -> Cow<[u32]> {
        Cow::Borrowed(input)
    }

    fn to_vec_f32(input: &[u32]) -> Cow<[f32]> {
        // TODO: there is a loss of precision this way,
        // this can be improved by going via i24 when that is
        // implemented.
        Cow::Owned(Sample::to_vec_f32(&Sample::to_vec_i16(input)).to_vec())
    }
}

impl Sample for f32 {
    fn get_format(_: Option<f32>) -> SampleFormat {
        SampleFormat::F32
    }

    fn interpolate(self, other: f32) -> f32 {
        (self + other) / 2.0
    }

    fn to_vec_i16(input: &[f32]) -> Cow<[i16]> {
        Cow::Owned(input.iter().map(|&value| {
            if value >= 0.0 {
                (value * 32767.0) as i16
            } else {
                (value * 32768.0) as i16
            }
        }).collect())
    }

    fn to_vec_u16(input: &[f32]) -> Cow<[u16]> {
        Cow::Owned(input.iter().map(|&value| {
            if value >= 0.0 {
                ((value * 32767.0) + 32768.0) as u16
            } else {
                ((value * 32768.0) + 32768.0) as u16
            }
        }).collect())
    }

    fn to_vec_u24(input: &[f32]) -> Cow<[u32]> {
        Cow::Owned(input.iter().map(|&value| {
            if value >= 0.0 {
                (value * 8388607.0) as u32 + 0x800000
            } else {
                (value * 8388608.0) as u32 + 0x800000
            }
        }).collect())
    }

    fn to_vec_f32(input: &[f32]) -> Cow<[f32]> {
        Cow::Borrowed(input)
    }
}

#[cfg(test)]
mod test {
    use super::Sample;

    #[test]
    fn i16_to_i16() {
        let out = Sample::to_vec_i16(&[0i16, -467, 32767, -32768]).into_owned();
        assert_eq!(out, vec![0, -467, 32767, -32768]);
    }

    #[test]
    fn i16_to_u16() {
        let out = Sample::to_vec_u16(&[0i16, -16384, 32767, -32768]).into_owned();
        assert_eq!(out, vec![32768, 16384, 65535, 0]);
    }

    #[test]
    fn i16_to_f32() {
        let out = Sample::to_vec_f32(&[0i16, -16384, 32767, -32768]).into_owned();
        assert_eq!(out, vec![0.0, -0.5, 1.0, -1.0]);
    }

    #[test]
    fn u16_to_i16() {
        let out = Sample::to_vec_i16(&[32768u16, 16384, 65535, 0]).into_owned();
        assert_eq!(out, vec![0, -16384, 32767, -32768]);
    }

    #[test]
    fn u16_to_u16() {
        let out = Sample::to_vec_u16(&[0u16, 467, 32767, 65535]).into_owned();
        assert_eq!(out, vec![0, 467, 32767, 65535]);
    }

    #[test]
    fn u16_to_f32() {
        let out = Sample::to_vec_f32(&[0u16, 32768, 65535]).into_owned();
        assert_eq!(out, vec![-1.0, 0.0, 1.0]);
    }

    #[test]
    fn f32_to_i16() {
        let out = Sample::to_vec_i16(&[0.0f32, -0.5, 1.0, -1.0]).into_owned();
        assert_eq!(out, vec![0, -16384, 32767, -32768]);
    }

    #[test]
    fn f32_to_u16() {
        let out = Sample::to_vec_u16(&[-1.0f32, 0.0, 1.0]).into_owned();
        assert_eq!(out, vec![0, 32768, 65535]);
    }

    #[test]
    fn f32_to_f32() {
        let out = Sample::to_vec_f32(&[0.1f32, -0.7, 1.0]).into_owned();
        assert_eq!(out, vec![0.1, -0.7, 1.0]);
    }
}
