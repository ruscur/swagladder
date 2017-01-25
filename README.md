swagladder
==========

swagladder is a free software web app for managing competitive ladders with
ELO rankings and matchmaking.

It is written in Rust because it's my favourite language at the moment.

It is called swagladder because it's a unique name and swag is fun to say.

Contributing
------------

Raise issues and send PRs.  Please one commit per logical change and sign
your commits (`git commit -s`) indicating that you agree to the
[Developer Certificate of Origin v1.1](http://developercertificate.org).

swagladder is licensed GPLv3 (see LICENSE) - you can use it and change it
and all that good stuff but you must release your changes to the world.

Dependencies
------------

You need Rust, Cargo (which you should have if you have Rust) and a local
redis instance.

I run Arch Linux, so the whole process would look something like this:

```
git clone https://github.com/ruscur/swagladder.git
cd swagladder
sudo pacman -S rust cargo redis
sudo systemctl start redis
cargo run
```

For other Linux distributions (or heaven forbid, other operating systems),
figure it out yourself.  It's probably a little harder than that.

Usage
-----

swagladder comes up at 127.0.0.1:42069 (lol).  For "production" you will
want to put a reverse proxy in front of that, like Nginx.

There's no UI for adding players or results yet, it's all a plain HTTP
API.  That's still super volatile so I'm not gonna document it atm.

