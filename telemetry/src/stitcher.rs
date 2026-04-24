use std::{io::{Read, Seek, Write}, usize};

use crate::archive::FragmentBody;

pub fn stitch_all<const N: usize, R, W, B>(
    inputs: &mut [&mut R],
    output: &mut W,
)
where
    R: Read + Seek,
    W: Write + Seek,
    B: FragmentBody<N>
{
    // parse all archive headers
    // 

    // iterate each buffer type
    for buf_type in B::LAYOUT {
        
    }

    // implementation
}

pub fn merge() {
    
}