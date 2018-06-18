# Eve: Environment editor
[![Linux build status](https://img.shields.io/travis/Aaronepower/eve.svg?branch=master)](https://travis-ci.org/Aaronepower/eve)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/Aaronepower/eve?svg=true)](https://ci.appveyor.com/project/Aaronepower/eve)
[![](https://img.shields.io/crates/d/eve.svg)](https://crates.io/crates/eve)
[![](https://img.shields.io/github/issues-raw/Aaronepower/eve.svg)](https://github.com/Aaronepower/eve/issues)
[![](https://tokei.rs/b1/github/Aaronepower/eve?category=code)](https://github.com/Aaronepower/eve)
[![Documentation](https://docs.rs/eve/badge.svg)](https://docs.rs/eve/)
[![Donate using Liberapay](https://liberapay.com/assets/widgets/donate.svg)](https://liberapay.com/Aaronepower/donate)

The **eve** utility reads the specified files, or standard input if no files are
specified, replacing all instances of `{{VAR}}` with the environment variable
of the same name e.g. `$VAR`. This utility is mainly useful as a replacement to
using `sed` to insert environment variables into files. As is common when using
Docker.

## Installation

#### Binary

###### Automatic
```
cargo install eve
```

###### Manual
You can download prebuilt binaries in the [releases section] or create one
from source.

```shell
$ git clone https://github.com/Aaronepower/eve.git
$ cd eve
$ cargo build --release
```
###### Linux/OSX
```
# sudo mv target/release/eve /usr/local/bin
```
###### Windows
- Create a folder for eve
- search for `env`
- open "edit your enviroment variables"
- edit `PATH`
- append folder path to the end of the string ie: `<path>;C:/eve/;`

#### Library
```
eve = "0.1"
```

## Example
Here's an example of replacing variables in a nginx configuration with
environment variables, and comparsion with the equivalent `sed` command.

#### `nginx.conf`
```nginx
server {
    listen 80;
    listen [::]:80;

    server_name {{NGINX_HOST}};

    location / {
        proxy_pass {{NGINX_PROXY}};
        proxy_next_upstream error timeout invalid_header http_500 http_502
            http_503 http_504;
        proxy_redirect off;
        proxy_buffering off;
        proxy_set_header        Host            {{NGINX_HOST}};
        proxy_set_header        X-Real-IP       $remote_addr;
        proxy_set_header        X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

#### `.env`
```
NGINX_HOST=localhost
NGINX_PROXY=localhost:8000
```

#### Commands

###### `sed`
```bash
sed -e "s|{{NGINX_HOST}}|$NGINX_HOST|" \
    -e "s|{{NGINX_PROXY}}|$NGINX_PROXY|" \
    nginx.conf
```
###### `eve`
```bash
eve nginx.conf
```

#### Output
```bash
server {
    listen 80;
    listen [::]:80;

    server_name localhost;

    location / {
        proxy_pass localhost:8000;
        proxy_next_upstream error timeout invalid_header http_500 http_502
            http_503 http_504;
        proxy_redirect off;
        proxy_buffering off;
        proxy_set_header        Host            localhost;
        proxy_set_header        X-Real-IP       $remote_addr;
        proxy_set_header        X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

[releases section]: https://github.com/Aaronepower/eve/releases
