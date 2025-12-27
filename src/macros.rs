macro_rules! guid {
    ($value:literal) => {
        crate::shared::guid::Guid::from_str($value).unwrap()
    };
}

macro_rules! exguid {
    ({{$guid:tt} , $n:literal}) => {
        crate::fsshttpb::data::exguid::ExGuid::from_guid(guid!($guid), $n)
    };
}

#[cfg(test)]
mod test {
    use crate::fsshttpb::data::exguid::ExGuid;
    use crate::shared::guid::Guid;

    #[test]
    fn parse_guid() {
        let guid = guid!("1A5A319C-C26B-41AA-B9C5-9BD8C44E07D4");

        assert_eq!(
            guid,
            Guid::from_str("1A5A319C-C26B-41AA-B9C5-9BD8C44E07D4").unwrap()
        );
    }

    #[test]
    fn parse_exguid() {
        let guid = exguid!({{"1A5A319C-C26B-41AA-B9C5-9BD8C44E07D4"}, 1});

        assert_eq!(
            guid,
            ExGuid::from_guid(
                Guid::from_str("1A5A319C-C26B-41AA-B9C5-9BD8C44E07D4").unwrap(),
                1
            )
        );
    }
}
