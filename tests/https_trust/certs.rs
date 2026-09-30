use std::fs;
use std::path::Path;
use std::process::Command;

struct LeafSpec<'a> {
    ca_pem: &'a str,
    ca_key: &'a str,
    out: &'a str,
    ext: &'a str,
    validity: &'a [&'a str],
}

const DAYS: &[&str] = &["-days", "2"];
const EXPIRED: &[&str] = &[
    "-not_before",
    "20200101000000Z",
    "-not_after",
    "20200102000000Z",
];

pub(super) fn build_certs(dir: &Path) {
    make_ca(dir, "ca.key", "ca.pem", "malvin-test-ca");
    make_ca(dir, "other.key", "other.pem", "malvin-other-ca");
    run_openssl(
        dir,
        &[
            "req",
            "-newkey",
            "ec",
            "-pkeyopt",
            "ec_paramgen_curve:prime256v1",
            "-nodes",
            "-keyout",
            "server.key",
            "-out",
            "server.csr",
            "-subj",
            "/CN=127.0.0.1",
        ],
    );
    write_san_ext(dir, "good.ext", "IP:127.0.0.1");
    write_san_ext(dir, "wrong.ext", "DNS:wrong.example");
    sign_leaf(
        dir,
        &leaf(["ca.pem", "ca.key", "good.pem", "good.ext"], DAYS),
    );
    sign_leaf(
        dir,
        &leaf(["ca.pem", "ca.key", "wrong.pem", "wrong.ext"], DAYS),
    );
    sign_leaf(
        dir,
        &leaf(["ca.pem", "ca.key", "expired.pem", "good.ext"], EXPIRED),
    );
    sign_leaf(
        dir,
        &leaf(
            ["other.pem", "other.key", "untrusted.pem", "good.ext"],
            DAYS,
        ),
    );
}

const fn leaf<'a>(names: [&'a str; 4], validity: &'a [&'a str]) -> LeafSpec<'a> {
    LeafSpec {
        ca_pem: names[0],
        ca_key: names[1],
        out: names[2],
        ext: names[3],
        validity,
    }
}

fn make_ca(dir: &Path, key: &str, pem: &str, cn: &str) {
    let subject = format!("/CN={cn}");
    run_openssl(
        dir,
        &[
            "req",
            "-x509",
            "-newkey",
            "ec",
            "-pkeyopt",
            "ec_paramgen_curve:prime256v1",
            "-nodes",
            "-keyout",
            key,
            "-out",
            pem,
            "-days",
            "2",
            "-subj",
            &subject,
            "-addext",
            "basicConstraints=critical,CA:TRUE",
            "-addext",
            "keyUsage=critical,keyCertSign,cRLSign",
        ],
    );
}

fn write_san_ext(dir: &Path, name: &str, san: &str) {
    let body =
        format!("subjectAltName={san}\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n");
    fs::write(dir.join(name), body).expect("write extension");
}

fn sign_leaf(dir: &Path, spec: &LeafSpec<'_>) {
    let mut args = vec![
        "x509",
        "-req",
        "-in",
        "server.csr",
        "-CA",
        spec.ca_pem,
        "-CAkey",
        spec.ca_key,
        "-CAcreateserial",
        "-out",
        spec.out,
        "-extfile",
        spec.ext,
    ];
    args.extend_from_slice(spec.validity);
    run_openssl(dir, &args);
}

fn run_openssl(dir: &Path, args: &[&str]) {
    let output = Command::new("openssl")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("openssl");
    assert!(
        output.status.success(),
        "openssl failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
