pub use gecl::{Point, Size};

pub const DEFAULT_DPI: i32 = 96;

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Logical<T>(pub T);

impl<T> std::ops::Deref for Logical<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Logical<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Physical<T>(pub T);

impl<T> std::ops::Deref for Physical<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Physical<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Screen<T>(pub T);

impl<T> std::ops::Deref for Screen<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Screen<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[inline]
fn to_physical_value<T>(a: T, dpi: T) -> T
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    a * dpi / num::cast(DEFAULT_DPI).unwrap()
}

#[inline]
fn to_logical_value<T>(a: T, dpi: T) -> T
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    a * num::cast(DEFAULT_DPI).unwrap() / dpi
}

impl<T> Logical<Point<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    #[inline]
    pub fn cast<U>(&self) -> Option<Logical<Point<U>>>
    where
        U: num::NumCast,
    {
        self.0.cast().map(|v| Logical(v))
    }

    #[inline]
    pub fn to_physical(&self, dpi: T) -> Physical<Point<T>> {
        Physical(Point::new(
            to_physical_value(self.x, dpi),
            to_physical_value(self.y, dpi),
        ))
    }
}

impl<T> Logical<Size<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    #[inline]
    pub fn cast<U>(&self) -> Option<Logical<Size<U>>>
    where
        U: num::NumCast,
    {
        self.0.cast().map(|v| Logical(v))
    }

    #[inline]
    pub fn to_physical(&self, dpi: T) -> Physical<Size<T>> {
        Physical(Size::new(
            to_physical_value(self.width, dpi),
            to_physical_value(self.height, dpi),
        ))
    }
}

impl<T> Physical<Point<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    #[inline]
    pub fn cast<U>(&self) -> Option<Physical<Point<U>>>
    where
        U: num::NumCast,
    {
        self.0.cast().map(|v| Physical(v))
    }

    #[inline]
    pub fn to_logical(&self, dpi: T) -> Logical<Point<T>> {
        Logical(Point::new(
            to_logical_value(self.x, dpi),
            to_logical_value(self.y, dpi),
        ))
    }
}

impl<T> Physical<Size<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    #[inline]
    pub fn cast<U>(&self) -> Option<Physical<Size<U>>>
    where
        U: num::NumCast,
    {
        self.0.cast().map(|v| Physical(v))
    }

    #[inline]
    pub fn to_logical(&self, dpi: T) -> Logical<Size<T>> {
        Logical(Size::new(
            to_logical_value(self.width, dpi),
            to_logical_value(self.height, dpi),
        ))
    }
}

impl<T> Screen<Point<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    #[inline]
    pub fn cast<U>(&self) -> Option<Screen<Point<U>>>
    where
        U: num::NumCast,
    {
        self.0.cast().map(|v| Screen(v))
    }
}

pub type LogicalPoint<T> = Logical<Point<T>>;
pub type LogicalSize<T> = Logical<Size<T>>;
pub type PhysicalPoint<T> = Physical<Point<T>>;
pub type PhysicalSize<T> = Physical<Size<T>>;
pub type ScreenPoint<T> = Screen<Point<T>>;

pub trait ToLogical {
    type Output;
    type Value;

    fn to_logical(&self, dpi: Self::Value) -> Logical<Self::Output>;
}

pub trait ToPhysical {
    type Output;
    type Value;

    fn to_physical(&self, dpi: Self::Value) -> Physical<Self::Output>;
}

impl<T> ToLogical for Logical<Point<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Point<T>;
    type Value = T;

    #[inline]
    fn to_logical(&self, _dpi: Self::Value) -> Logical<Self::Output> {
        *self
    }
}

impl<T> ToLogical for Physical<Point<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Point<T>;
    type Value = T;

    #[inline]
    fn to_logical(&self, dpi: Self::Value) -> Logical<Self::Output> {
        self.to_logical(dpi)
    }
}

impl<T> ToPhysical for Logical<Point<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Point<T>;
    type Value = T;

    #[inline]
    fn to_physical(&self, dpi: Self::Value) -> Physical<Self::Output> {
        self.to_physical(dpi)
    }
}

impl<T> ToPhysical for Physical<Point<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Point<T>;
    type Value = T;

    #[inline]
    fn to_physical(&self, _dpi: Self::Value) -> Physical<Self::Output> {
        *self
    }
}

impl<T> ToLogical for Logical<Size<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Size<T>;
    type Value = T;

    #[inline]
    fn to_logical(&self, _dpi: Self::Value) -> Logical<Self::Output> {
        *self
    }
}

impl<T> ToLogical for Physical<Size<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Size<T>;
    type Value = T;

    #[inline]
    fn to_logical(&self, dpi: Self::Value) -> Logical<Self::Output> {
        self.to_logical(dpi)
    }
}

impl<T> ToPhysical for Logical<Size<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Size<T>;
    type Value = T;

    #[inline]
    fn to_physical(&self, dpi: Self::Value) -> Physical<Self::Output> {
        self.to_physical(dpi)
    }
}

impl<T> ToPhysical for Physical<Size<T>>
where
    T: num::traits::NumOps + num::NumCast + Copy,
{
    type Output = Size<T>;
    type Value = T;

    #[inline]
    fn to_physical(&self, _dpi: Self::Value) -> Physical<Self::Output> {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast() {
        let src = Logical(Point::new(128i32, 256i32));
        let dest = src.cast::<u32>().unwrap();
        assert!(src.x as u32 == dest.x);
        assert!(src.y as u32 == dest.y);
    }

    #[test]
    fn logical_to_logical() {
        let src = Logical(Point::new(128, 256));
        let dest = src.to_logical(DEFAULT_DPI);
        assert!(src.x == dest.x);
        assert!(src.y == dest.y);
    }

    #[test]
    fn logical_to_physical() {
        let src = Logical(Point::new(128, 256));
        let dest = src.to_physical(DEFAULT_DPI * 2);
        assert!(src.x * 2 == dest.x);
        assert!(src.y * 2 == dest.y);
    }

    #[test]
    fn physical_to_physical() {
        let src = Physical(Point::new(128, 256));
        let dest = src.to_physical(DEFAULT_DPI);
        assert!(src.x == dest.x);
        assert!(src.y == dest.y);
    }

    #[test]
    fn physical_to_logical() {
        let src = Physical(Point::new(128, 256));
        let dest = src.to_logical(DEFAULT_DPI * 2);
        assert!(src.x == dest.x * 2);
        assert!(src.y == dest.y * 2);
    }
}
