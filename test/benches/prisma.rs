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

    let mut r = byte_reader::Reader::new(SCHEMA.as_bytes());
    assert_eq!(Schema::parse(&mut r), Schema {
        generator_client: GeneratorClient {
            provider: "qujila",
            output:   "../src/my_db_module",
        },
        datasource: Datasouce {
            name:     "db",
            provider: "postgresql",
            url:      "DATABASE_URL",
        }
    });

    b.iter(|| {
        let mut buf = Vec::with_capacity(10);
        for _ in 0..10 {
            let mut r = byte_reader::Reader::new(SCHEMA.as_bytes());
            buf.push(Schema::parse(&mut r))
        }
        buf
    })
}

mod prisma {
    use byte_reader::Reader;

    pub trait Parse<'p> {
        fn parse<'r>(r: &'r mut Reader<'p>) -> Self;
    }

    #[cfg(feature="text")]
    fn read_string<'r>(r: &mut Reader<'r>) -> Option<&'r str> {
        r.read_quoted_by(b'"', b'"')
            .map(|bytes| std::str::from_utf8(bytes).unwrap())
    }
    #[cfg(not(feature="text"))]
    fn read_string<'r>(r: &mut Reader<'r>) -> Option<&'r str> {
        r.consume("\"")?;
        let string = r.read_while(|b| b != &b'"');
        r.consume("\"").unwrap();
        Some(std::str::from_utf8(string).unwrap())
    }

    #[derive(Debug, PartialEq)]
    pub struct Schema<'s> {
        pub generator_client: GeneratorClient<'s>,
        pub datasource:       Datasouce<'s>,
    } impl<'p> Parse<'p> for Schema<'p> {
        fn parse<'r>(r: &'r mut Reader<'p>) -> Self {
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
    pub struct GeneratorClient<'s> {
        pub provider: &'s str,
        pub output:   &'s str,
    } impl<'p> Parse<'p> for GeneratorClient<'p> {
        fn parse<'r>(r: &'r mut Reader<'p>) -> Self {
            r.consume("generator").unwrap(); r.skip_whitespace();
            r.consume("client").unwrap();    r.skip_whitespace();
            r.consume("{").unwrap();         r.skip_whitespace();
            let (mut provider, mut output) = (None, None);
            while r.peek().is_some_and(|b| b != &b'}') {
                r.skip_whitespace();
                match r.consume_oneof(["provider", "output"]).unwrap() {
                    0 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        provider = Some(read_string(r).unwrap());
                    }
                    1 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        output = Some(read_string(r).unwrap());
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
    pub struct Datasouce<'d> {
        pub name:     &'d str,
        pub provider: &'d str,
        pub url:      &'d str,
    } impl<'p> Parse<'p> for Datasouce<'p> {
        fn parse<'r>(r: &'r mut Reader<'p>) -> Self {
            r.consume("datasource").unwrap();
            r.skip_whitespace();

            let name = std::str::from_utf8(
                r.read_while(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z'))
            ).unwrap();
            r.skip_whitespace();

            r.consume("{").unwrap();
            r.skip_whitespace();

            let (mut provider, mut url) = (None, None);
            while r.peek().is_some_and(|b| b != &b'}') {
                r.skip_whitespace();
                match r.consume_oneof(["provider", "url"]).unwrap() {
                    0 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        provider = Some(read_string(r).unwrap());
                    }
                    1 => {r.skip_whitespace();
                        r.consume("=").unwrap(); r.skip_whitespace();
                        r.consume("env").unwrap();
                        r.consume("(").unwrap();
                        url = Some(read_string(r).unwrap());
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
