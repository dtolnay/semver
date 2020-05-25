// This file imports semver 0.10.0's test suite, to protect against regressions.

// https://github.com/steveklabnik/semver/blob/afa5fc853cb4d6d2b1329579e5528f86f3b550f9/src/version.rs#L384-L871
mod version {
    #[test]
    fn test_parse() {
        assert!(!semver::valid(""));
        assert!(!semver::valid("  "));
        assert!(!semver::valid("1"));
        assert!(!semver::valid("1.2"));
        /*
        assert!(!semver::valid("1.2.3-"));
        assert!(!semver::valid("a.b.c"));
        assert!(!semver::valid("1.2.3 abc"));

        assert!(semver::valid("1.2.3"));
        assert!(semver::valid("  1.2.3  "));
        assert!(semver::valid("1.2.3-alpha1"));
        assert!(semver::valid("  1.2.3-alpha1  "));
        assert!(semver::valid("1.2.3+build5"));
        assert!(semver::valid("  1.2.3+build5  "));
        assert!(semver::valid("1.2.3-alpha1+build5"));
        assert!(semver::valid("  1.2.3-alpha1+build5  "));
        assert!(semver::valid("1.2.3-1.alpha1.9+build5.7.3aedf  "));
        assert!(semver::valid("0.4.0-beta.1+0851523"));
        */
    }
}

// https://github.com/steveklabnik/semver/blob/afa5fc853cb4d6d2b1329579e5528f86f3b550f9/src/version_req.rs#L594-L990
mod version_req {
    /*

        #[test]
        fn test_parsing_default() {
            let r = req("1.0.0");

            assert_eq!(r.to_string(), "^1.0.0".to_string());

            assert_match(&r, &["1.0.0", "1.0.1"]);
            assert_not_match(&r, &["0.9.9", "0.10.0", "0.1.0"]);
        }

        #[test]
        fn test_parsing_exact() {
            let r = req("=1.0.0");

            assert!(r.to_string() == "=1.0.0".to_string());
            assert_eq!(r.to_string(), "=1.0.0".to_string());

            assert_match(&r, &["1.0.0"]);
            assert_not_match(&r, &["1.0.1", "0.9.9", "0.10.0", "0.1.0", "1.0.0-pre"]);

            let r = req("=0.9.0");

            assert_eq!(r.to_string(), "=0.9.0".to_string());

            assert_match(&r, &["0.9.0"]);
            assert_not_match(&r, &["0.9.1", "1.9.0", "0.0.9"]);

            let r = req("=0.1.0-beta2.a");

            assert_eq!(r.to_string(), "=0.1.0-beta2.a".to_string());

            assert_match(&r, &["0.1.0-beta2.a"]);
            assert_not_match(&r, &["0.9.1", "0.1.0", "0.1.1-beta2.a", "0.1.0-beta2"]);
        }

        #[test]
        pub fn test_parsing_greater_than() {
            let r = req(">= 1.0.0");

            assert_eq!(r.to_string(), ">=1.0.0".to_string());

            assert_match(&r, &["1.0.0", "2.0.0"]);
            assert_not_match(&r, &["0.1.0", "0.0.1", "1.0.0-pre", "2.0.0-pre"]);

            let r = req(">= 2.1.0-alpha2");

            assert_match(&r, &["2.1.0-alpha2", "2.1.0-alpha3", "2.1.0", "3.0.0"]);
            assert_not_match(
                &r,
                &["2.0.0", "2.1.0-alpha1", "2.0.0-alpha2", "3.0.0-alpha2"],
            );
        }

        #[test]
        pub fn test_parsing_less_than() {
            let r = req("< 1.0.0");

            assert_eq!(r.to_string(), "<1.0.0".to_string());

            assert_match(&r, &["0.1.0", "0.0.1"]);
            assert_not_match(&r, &["1.0.0", "1.0.0-beta", "1.0.1", "0.9.9-alpha"]);

            let r = req("<= 2.1.0-alpha2");

            assert_match(&r, &["2.1.0-alpha2", "2.1.0-alpha1", "2.0.0", "1.0.0"]);
            assert_not_match(
                &r,
                &["2.1.0", "2.2.0-alpha1", "2.0.0-alpha2", "1.0.0-alpha2"],
            );
        }

        #[test]
        pub fn test_multiple() {
            let r = req("> 0.0.9, <= 2.5.3");
            assert_eq!(r.to_string(), ">0.0.9, <=2.5.3".to_string());
            assert_match(&r, &["0.0.10", "1.0.0", "2.5.3"]);
            assert_not_match(&r, &["0.0.8", "2.5.4"]);

            let r = req("0.3.0, 0.4.0");
            assert_eq!(r.to_string(), "^0.3.0, ^0.4.0".to_string());
            assert_not_match(&r, &["0.0.8", "0.3.0", "0.4.0"]);

            let r = req("<= 0.2.0, >= 0.5.0");
            assert_eq!(r.to_string(), "<=0.2.0, >=0.5.0".to_string());
            assert_not_match(&r, &["0.0.8", "0.3.0", "0.5.1"]);

            let r = req("0.1.0, 0.1.4, 0.1.6");
            assert_eq!(r.to_string(), "^0.1.0, ^0.1.4, ^0.1.6".to_string());
            assert_match(&r, &["0.1.6", "0.1.9"]);
            assert_not_match(&r, &["0.1.0", "0.1.4", "0.2.0"]);

            assert!(VersionReq::parse("> 0.1.0,").is_err());
            assert!(VersionReq::parse("> 0.3.0, ,").is_err());

            let r = req(">=0.5.1-alpha3, <0.6");
            assert_eq!(r.to_string(), ">=0.5.1-alpha3, <0.6".to_string());
            assert_match(
                &r,
                &[
                    "0.5.1-alpha3",
                    "0.5.1-alpha4",
                    "0.5.1-beta",
                    "0.5.1",
                    "0.5.5",
                ],
            );
            assert_not_match(
                &r,
                &["0.5.1-alpha1", "0.5.2-alpha3", "0.5.5-pre", "0.5.0-pre"],
            );
            assert_not_match(&r, &["0.6.0", "0.6.0-pre"]);
        }

        #[test]
        pub fn test_parsing_tilde() {
            let r = req("~1");
            assert_match(&r, &["1.0.0", "1.0.1", "1.1.1"]);
            assert_not_match(&r, &["0.9.1", "2.9.0", "0.0.9"]);

            let r = req("~1.2");
            assert_match(&r, &["1.2.0", "1.2.1"]);
            assert_not_match(&r, &["1.1.1", "1.3.0", "0.0.9"]);

            let r = req("~1.2.2");
            assert_match(&r, &["1.2.2", "1.2.4"]);
            assert_not_match(&r, &["1.2.1", "1.9.0", "1.0.9", "2.0.1", "0.1.3"]);

            let r = req("~1.2.3-beta.2");
            assert_match(&r, &["1.2.3", "1.2.4", "1.2.3-beta.2", "1.2.3-beta.4"]);
            assert_not_match(&r, &["1.3.3", "1.1.4", "1.2.3-beta.1", "1.2.4-beta.2"]);
        }

        #[test]
        pub fn test_parsing_compatible() {
            let r = req("^1");
            assert_match(&r, &["1.1.2", "1.1.0", "1.2.1", "1.0.1"]);
            assert_not_match(&r, &["0.9.1", "2.9.0", "0.1.4"]);
            assert_not_match(&r, &["1.0.0-beta1", "0.1.0-alpha", "1.0.1-pre"]);

            let r = req("^1.1");
            assert_match(&r, &["1.1.2", "1.1.0", "1.2.1"]);
            assert_not_match(&r, &["0.9.1", "2.9.0", "1.0.1", "0.1.4"]);

            let r = req("^1.1.2");
            assert_match(&r, &["1.1.2", "1.1.4", "1.2.1"]);
            assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
            assert_not_match(&r, &["1.1.2-alpha1", "1.1.3-alpha1", "2.9.0-alpha1"]);

            let r = req("^0.1.2");
            assert_match(&r, &["0.1.2", "0.1.4"]);
            assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
            assert_not_match(&r, &["0.1.2-beta", "0.1.3-alpha", "0.2.0-pre"]);

            let r = req("^0.5.1-alpha3");
            assert_match(
                &r,
                &[
                    "0.5.1-alpha3",
                    "0.5.1-alpha4",
                    "0.5.1-beta",
                    "0.5.1",
                    "0.5.5",
                ],
            );
            assert_not_match(
                &r,
                &[
                    "0.5.1-alpha1",
                    "0.5.2-alpha3",
                    "0.5.5-pre",
                    "0.5.0-pre",
                    "0.6.0",
                ],
            );

            let r = req("^0.0.2");
            assert_match(&r, &["0.0.2"]);
            assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1", "0.1.4"]);

            let r = req("^0.0");
            assert_match(&r, &["0.0.2", "0.0.0"]);
            assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.1.4"]);

            let r = req("^0");
            assert_match(&r, &["0.9.1", "0.0.2", "0.0.0"]);
            assert_not_match(&r, &["2.9.0", "1.1.1"]);

            let r = req("^1.4.2-beta.5");
            assert_match(
                &r,
                &["1.4.2", "1.4.3", "1.4.2-beta.5", "1.4.2-beta.6", "1.4.2-c"],
            );
            assert_not_match(
                &r,
                &[
                    "0.9.9",
                    "2.0.0",
                    "1.4.2-alpha",
                    "1.4.2-beta.4",
                    "1.4.3-beta.5",
                ],
            );
        }

        #[test]
        pub fn test_parsing_wildcard() {
            let r = req("");
            assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
            assert_not_match(&r, &[]);
            let r = req("*");
            assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
            assert_not_match(&r, &[]);
            let r = req("x");
            assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
            assert_not_match(&r, &[]);
            let r = req("X");
            assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
            assert_not_match(&r, &[]);

            let r = req("1.*");
            assert_match(&r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
            assert_not_match(&r, &["0.0.9"]);
            let r = req("1.x");
            assert_match(&r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
            assert_not_match(&r, &["0.0.9"]);
            let r = req("1.X");
            assert_match(&r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
            assert_not_match(&r, &["0.0.9"]);

            let r = req("1.2.*");
            assert_match(&r, &["1.2.0", "1.2.2", "1.2.4"]);
            assert_not_match(&r, &["1.9.0", "1.0.9", "2.0.1", "0.1.3"]);
            let r = req("1.2.x");
            assert_match(&r, &["1.2.0", "1.2.2", "1.2.4"]);
            assert_not_match(&r, &["1.9.0", "1.0.9", "2.0.1", "0.1.3"]);
            let r = req("1.2.X");
            assert_match(&r, &["1.2.0", "1.2.2", "1.2.4"]);
            assert_not_match(&r, &["1.9.0", "1.0.9", "2.0.1", "0.1.3"]);
        }

        #[test]
        pub fn test_any() {
            let r = VersionReq::any();
            assert_match(&r, &["0.0.1", "0.1.0", "1.0.0"]);
        }

        #[test]
        pub fn test_pre() {
            let r = req("=2.1.1-really.0");
            assert_match(&r, &["2.1.1-really.0"]);
        }

        #[test]
        pub fn test_from_str() {
            assert_eq!(
                "1.0.0".parse::<VersionReq>().unwrap().to_string(),
                "^1.0.0".to_string()
            );
            assert_eq!(
                "=1.0.0".parse::<VersionReq>().unwrap().to_string(),
                "=1.0.0".to_string()
            );
            assert_eq!(
                "~1".parse::<VersionReq>().unwrap().to_string(),
                "~1".to_string()
            );
            assert_eq!(
                "~1.2".parse::<VersionReq>().unwrap().to_string(),
                "~1.2".to_string()
            );
            assert_eq!(
                "^1".parse::<VersionReq>().unwrap().to_string(),
                "^1".to_string()
            );
            assert_eq!(
                "^1.1".parse::<VersionReq>().unwrap().to_string(),
                "^1.1".to_string()
            );
            assert_eq!(
                "*".parse::<VersionReq>().unwrap().to_string(),
                "*".to_string()
            );
            assert_eq!(
                "1.*".parse::<VersionReq>().unwrap().to_string(),
                "1.*".to_string()
            );
            assert_eq!(
                "< 1.0.0".parse::<VersionReq>().unwrap().to_string(),
                "<1.0.0".to_string()
            );
        }

        // #[test]
        // pub fn test_from_str_errors() {
        //    assert_eq!(Err(InvalidVersionRequirement), "\0".parse::<VersionReq>());
        //    assert_eq!(Err(OpAlreadySet), ">= >= 0.0.2".parse::<VersionReq>());
        //    assert_eq!(Err(InvalidSigil), ">== 0.0.2".parse::<VersionReq>());
        //    assert_eq!(Err(VersionComponentsMustBeNumeric),
        //               "a.0.0".parse::<VersionReq>());
        //    assert_eq!(Err(InvalidIdentifier), "1.0.0-".parse::<VersionReq>());
        //    assert_eq!(Err(MajorVersionRequired), ">=".parse::<VersionReq>());
        // }

        #[test]
        fn test_cargo3202() {
            let v = "0.*.*".parse::<VersionReq>().unwrap();
            assert_eq!("0.*.*", format!("{}", v.predicates[0]));

            let v = "0.0.*".parse::<VersionReq>().unwrap();
            assert_eq!("0.0.*", format!("{}", v.predicates[0]));

            let r = req("0.*.*");
            assert_match(&r, &["0.5.0"]);
        }
    }
    */
}
