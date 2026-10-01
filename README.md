# Intro

Welcome to the `lehmah` mono-repro. The majority of products and services are intended to 
be contained in this repo. Where everything is written in pure-rust from top to bottom.


## Services
- [Oxypress](#oxypress)
- [Blackbot](#blackbot)
- [Closing Statements](#closing-statements)


# Oxypress

A lightweight blogging service written in Rust.

## Purpose

I needed a way to publish work online and thought I would be a good exercise to create my own service to suit my 
own needs. 

##  My needs are as followed: 

1. The server has to be **SMALL**. I'm renting a small VPS. Deploying with docker & k8's is overkill. It needs to be monolithic. Running this on a vps should be as simple as copying binaries,certs,static files, and executing the binary.

2. It needs to support LATEX and have a basic but modern "document builder". The publishing tool to make blog posts should look like something
similar to word. The same tool should also be used for comments. 

3. Needs **user monitoring**. I want to be able to see what people are reading, what they're engaging with, why. This is a sub-project that
requires a whole seperate REPO and is something that is lower priority at the momment.

# Blackbot 

A pure rust fully featured image-board. Currently *not* in the mono-repro, but it will be absorbed into the repo over time. 

## My needs are as followed:
1. Its gonna need `SQLX` integration
2. The front-end is garbage, I wrote much of it with `handlebars-rs` which creates
a lot of source of truth issues. 
3. Many of the database operations aren't written with transactions in mind, which causes DB corruption
4. I implemented timestamps in the worst way imaginable


# Closing statements

[Blackbot](#blackbot) is much lower priority than [Oxypress](#oxypress) at the momment. Top priority is:

1. Developing the `lehmah` infastructure so deployment is easy.
2. Developing [Oxypress](#oxypress) so that i'm visible on the web.

