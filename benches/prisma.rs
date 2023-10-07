#![feature(test)]
extern crate test;

const SCHEMA: &str = r#"
generator client {
    provider = "qujila"
    output   = "../src/my_db_module"
}

datasource db {
    provider = "postgresql"
    url      = env("DATABASE_URL")
}
"#;

#[bench] fn read_a_schema_dot_prisma(b: &mut test::Bencher) {
    use prisma::*;
    use std::format as f;

    b.iter(|| {
        let mut r = byte_reader::Reader::new(SCHEMA);
        assert_eq!(Schema::parse(&mut r), Schema {
            generator_client: GeneratorClient {
                provider: f!("qujila"),
                output:   f!("../src/my_db_module"),
            },
            datasource: Datasouce {
                name:     f!("db"),
                provider: f!("postgresql"),
                url:      f!("DATABASE_URL"),
            }
        })
    })
}

mod prisma {
    pub trait Parse {
        fn parse(r: &mut byte_reader::Reader) -> Self;
    }

    #[derive(Debug, PartialEq)]
    pub struct Schema {
        pub generator_client: GeneratorClient,
        pub datasource:       Datasouce,
    } impl Parse for Schema {
        fn parse(r: &mut byte_reader::Reader) -> Self {
            r.skip_whitespace();
            let (mut g, mut d) = (None, None);
            while let Some(next) = r.peek() {
                match next {
                    b'g' => g = Some(GeneratorClient::parse(r)),
                    b'd' => d = Some(Datasouce::parse(r)),
                    _ => unreachable!(),
                }
                r.skip_whitespace();
            }

            Self {
                generator_client: g.unwrap(),
                datasource:       d.unwrap(),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct GeneratorClient {
        pub provider: String,
        pub output:   String,
    } impl Parse for GeneratorClient {
        fn parse(r: &mut byte_reader::Reader) -> Self {
            r.consume("generator").unwrap(); r.skip_whitespace();
            r.consume("client").unwrap();    r.skip_whitespace();
            r.consume("{").unwrap();         r.skip_whitespace();
            let (mut provider, mut output) = (None, None);
            while r.peek().is_some_and(|b| b != &b'}') {
                r.skip_whitespace();
                match r.consume_oneof(["provider", "output"]).unwrap() {
                    0 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        provider = Some(r.read_string().unwrap());
                    }
                    1 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        output = Some(r.read_string().unwrap());
                    }
                    _ => unreachable!(),
                }
                r.skip_whitespace();
            }
            r.consume("}").unwrap();

            Self {
                provider: provider.unwrap(),
                output:   output.unwrap(),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Datasouce {
        pub name:     String,
        pub provider: String,
        pub url:      String,
    } impl Parse for Datasouce {
        fn parse(r: &mut byte_reader::Reader) -> Self {
            r.consume("datasource").unwrap();   r.skip_whitespace();
            let name = r.read_snake().unwrap(); r.skip_whitespace();
            r.consume("{").unwrap();            r.skip_whitespace();
            let (mut provider, mut url) = (None, None);
            while r.peek().is_some_and(|b| b != &b'}') {
                r.skip_whitespace();
                match r.consume_oneof(["provider", "url"]).unwrap() {
                    0 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        provider = Some(r.read_string().unwrap());
                    }
                    1 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        r.consume("env").unwrap();
                        r.consume("(").unwrap();
                        url = Some(r.read_string().unwrap());
                        r.consume(")");
                    }
                    _ => unreachable!(),
                }
                r.skip_whitespace();
            }
            r.consume("}").unwrap();

            Self {
                name,
                provider: provider.unwrap(),
                url:      url.unwrap(),
            }
        }
    }
}
