use core::fmt::Debug;

use self::point::{AffinePoint, AffinePointRegister};
use super::builder::AirBuilder;
use super::field::parameters::FieldParameters;
use super::AirParameters;

pub mod edwards;
pub mod gadget;
pub mod point;
pub mod scalar_mul;
pub mod weierstrass;

pub trait EllipticCurveParameters: Debug + Send + Sync + Copy + 'static {
    type BaseField: FieldParameters;
}

/// An interface for elliptic curve groups.
pub trait EllipticCurve: EllipticCurveParameters {
    /// Adds two different points on the curve.
    ///
    /// Warning: This method assumes that the two points are different.
    fn ec_add(p: &AffinePoint<Self>, q: &AffinePoint<Self>) -> AffinePoint<Self>;

    /// Doubles a point on the curve.
    fn ec_double(p: &AffinePoint<Self>) -> AffinePoint<Self>;

    /// Returns the generator of the curve group for a curve/subgroup of prime order.
    fn ec_generator() -> AffinePoint<Self>;

    /// Returns the neutral element of the curve group, if this element is affine (such as in the
    /// case of the Edwards curve group). Otherwise, returns `None`.
    fn ec_neutral() -> Option<AffinePoint<Self>>;

    /// Returns the negative of a point on the curve.
    fn ec_neg(p: &AffinePoint<Self>) -> AffinePoint<Self>;

    /// Returns the number of bits needed to represent a scalar in the group.
    fn nb_scalar_bits() -> usize {
        Self::BaseField::NB_LIMBS * Self::BaseField::NB_BITS_PER_LIMB
    }
}

pub trait EllipticCurveAir<L: AirParameters>: EllipticCurve {
    fn ec_add_air(
        builder: &mut AirBuilder<L>,
        p: &AffinePointRegister<Self>,
        q: &AffinePointRegister<Self>,
    ) -> AffinePointRegister<Self>;

    fn ec_double_air(
        builder: &mut AirBuilder<L>,
        p: &AffinePointRegister<Self>,
    ) -> AffinePointRegister<Self>;

    fn ec_generator_air(builder: &mut AirBuilder<L>) -> AffinePointRegister<Self>;
}

impl<L: AirParameters> AirBuilder<L> {
    pub fn ec_add<E: EllipticCurveAir<L>>(
        &mut self,
        p: &AffinePointRegister<E>,
        q: &AffinePointRegister<E>,
    ) -> AffinePointRegister<E> {
        E::ec_add_air(self, p, q)
    }

    pub fn ec_double<E: EllipticCurveAir<L>>(
        &mut self,
        p: &AffinePointRegister<E>,
    ) -> AffinePointRegister<E> {
        E::ec_double_air(self, p)
    }

    pub fn ec_generator<E: EllipticCurveAir<L>>(&mut self) -> AffinePointRegister<E> {
        E::ec_generator_air(self)
    }
}
