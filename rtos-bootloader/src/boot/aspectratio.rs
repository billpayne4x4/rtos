#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AspectRatio {
    // Widescreen modern standards
    Ratio16_9,
    Ratio21_9,
    Ratio32_9,
    Ratio16_10,

    // Classic / legacy
    Ratio4_3,
    Ratio5_4,
    Ratio3_2,

    // Mobile / tablets
    Ratio18_9,
    Ratio19_9,
    Ratio20_9,

    // Cinematic & professional
    Ratio17_9,
    Ratio2_1,

    // Sentinel: no more fallback
    Unspecified,
}

impl AspectRatio {
    /// Returns (numerator, denominator) if defined; None for Unspecified.
    pub fn as_tuple(&self) -> Option<(u32, u32)> {
        use AspectRatio::*;
        match self {
            Ratio16_9   => Some((16, 9)),
            Ratio21_9   => Some((21, 9)),
            Ratio32_9   => Some((32, 9)),
            Ratio16_10  => Some((16, 10)),
            Ratio4_3    => Some((4, 3)),
            Ratio5_4    => Some((5, 4)),
            Ratio3_2    => Some((3, 2)),
            Ratio18_9   => Some((18, 9)),
            Ratio19_9   => Some((19, 9)),
            Ratio20_9   => Some((20, 9)),
            Ratio17_9   => Some((17, 9)),
            Ratio2_1    => Some((2, 1)),
            Unspecified => None,
        }
    }

    /// Update self to the next best fallback and return its tuple.
    /// Returns None when no fallback remains (self becomes Unspecified).
    pub fn get_fallback(&mut self) -> Option<(u32, u32)> {
        use AspectRatio::*;
        *self = match *self {
            // widescreen family
            Ratio32_9  => Ratio21_9,
            Ratio21_9  => Ratio16_9,
            Ratio16_10 => Ratio16_9,
            Ratio2_1   => Ratio16_9,
            Ratio17_9  => Ratio16_9,

            // classic family
            Ratio5_4 => Ratio4_3,
            Ratio3_2 => Ratio4_3,

            // mobile family
            Ratio20_9 => Ratio19_9,
            Ratio19_9 => Ratio18_9,
            Ratio18_9 => Ratio16_9,

            // bases fall off the chain
            Ratio16_9 | Ratio4_3 | Unspecified => Unspecified,
        };

        self.as_tuple()
    }
}
