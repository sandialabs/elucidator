use crate::designation::DesignationSpecification;
use crate::error::ElucidatorError;

pub trait Interpret: Sized {
    fn get_spec() -> DesignationSpecification;
    fn from_eluciblob() -> Result<Self, ElucidatorError>;
    fn to_eluciblob() -> Result<Self, ElucidatorError>;
}

#[cfg(test)]
mod test {
    use super::*;
    use elucidator_macros::Interpret;

    #[derive(Interpret, Debug, Clone)]
    struct MyPoint {
        x: u8,
        y: u8,
    }

    #[derive(Interpret, Debug, Clone)]
    struct MyPoint16 {
        x: u16,
        y: u16,
    }

    mod interpret {
        use super::*;

        // #[test]
        fn fails() {
            let p = MyPoint { x: 0, y: 1 };
            assert_eq!(0, 1);
        }

        #[test]
        fn get_spec_ok() {
            let spec = MyPoint::get_spec();
            let buffer: [u8; 2] = [0x00, 0x01];
            let gt_point = MyPoint { x: 0, y: 1 };
            let interpretted = spec.interpret(&buffer).unwrap();
            assert_eq!(interpretted.get("x").unwrap().as_u8().unwrap(), gt_point.x);
            assert_eq!(interpretted.get("y").unwrap().as_u8().unwrap(), gt_point.y);
        }

        #[test]
        fn get_spec_alt_ok() {
            let spec = MyPoint16::get_spec();
            let buffer: [u8; 4] = [0x0, 0x00, 0x01, 0x00];
            let gt_point = MyPoint16 { x: 0, y: 1 };
            let interpretted = spec.interpret(&buffer).unwrap();
            assert_eq!(
                interpretted.get("x").unwrap().as_u16().unwrap(),
                gt_point.x
            );
            assert_eq!(
                interpretted.get("y").unwrap().as_u16().unwrap(),
                gt_point.y
            );
        }
    }
}
