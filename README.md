# Oxypress
A lightweight blogging service written in Rust.

# Purpose

I needed a way to publish work online and thought I would be a good exercise to create my own service to suit my 
own needs. 

##  My needs are as followed: 

1. The server has to be **SMALL**. I'm renting a small VPS. Deploying with docker & k8's is overkill. 
2. It needs to support LATEX and have a basic but modern "document builder". The publishing tool to make blog posts should look like something
similar to word. The same tool should also be used for comments. 
3. Needs **user monitoring**. I want to be able to see what people are reading, what they're engaging with, why. This is a sub-project that
requires a whole seperate REPO and is something that is lower priority at the momment.
