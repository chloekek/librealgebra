use core::ops::Add;

/// A De Bruijn index references a variable.
#[derive(Clone, Copy, Debug)]
pub struct DeBruijn(pub u32);

impl Add<u32> for DeBruijn
{
    type Output = DeBruijn;

    fn add(self, rhs: u32) -> Self::Output
    {
        DeBruijn(self.0 + rhs)
    }
}
