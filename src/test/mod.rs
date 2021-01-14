#[cfg(test)]
mod tests {
    use crate::{check_configuration, Configuration, ProbeResult};

    #[test]
    fn test1() {
        do_test(
            "
            * 2 2 2 2 *
            2 _ 2 * * 3 
            _ _ _ _ * 3 
            _ _ ? _ _ _ 
            2 _ _ _ 4 2 
            * 3 3 _ _ _
        ",
            ProbeResult::Unknown,
        )
    }

    #[test]
    fn test2() {
        do_test(
            "
            _ _ 2 _ 3 _
            2 _ _ * * 3 
            1 1 2 4 _ 3 
            1 ? 3 4 _ 2 
            2 * * * _ 3 
            _ 3 3 3 * *
        ",
            ProbeResult::Safe,
        )
    }

    #[test]
    fn test3() {
        do_test(
            "
            _ _ 2 _ 3 _
            2 _ _ * * 3 
            1 1 2 4 _ 3 
            1 _ 3 4 _ 2 
            2 * ? * _ 3 
            _ 3 3 3 * *
        ",
            ProbeResult::Unsafe,
        )
    }

    #[test]
    fn test4() {
        do_test(
            "
            * 2 2 2 3 *
            2 _ 2 * * 3 
            1 1 2 4 * _ 
            1 2 3 4 _ ? 
            2 _ * * 4 3 
            * 3 3 3 * *
        ",
            ProbeResult::Safe,
        )
    }

    #[test]
    fn test_full() {
        do_test(
            "
            * 2 2 2 2 *
            2 * 2 * ? 3 
            1 1 2 4 * 3 
            1 2 3 4 * 2 
            2 * * * 4 2 
            * 3 3 3 * *
        ",
            ProbeResult::Unsafe,
        )
    }

    fn do_test(raw_conf: &str, is_safe: ProbeResult) {
        let conf = Configuration::from(raw_conf.trim().to_string());
        let result = check_configuration(&conf);
        assert_eq!(result, is_safe);
    }
}
