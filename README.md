# fuzzing-101-solutions

Companion repository to the [Fuzzing101 with LibAFL](https://epi052.gitlab.io/notes-to-self/blog/2021-11-01-fuzzing-101-with-libafl/) series of blog posts.

## Overview

Twitter user [Antonio Morales](https://twitter.com/nosoynadiemas?lang=en) created the [Fuzzing101](https://github.com/antonio-morales/Fuzzing101) repository 
in August of 2021. In the repo, he has created exercises and solutions meant to teach the basics of fuzzing to anyone who wants to learn how to find 
vulnerabilities in real software projects. The repo focuses on [AFL++](https://github.com/AFLplusplus/AFLplusplus) usage, but this repository aims to solve
the exercises using [LibAFL](https://github.com/AFLplusplus/LibAFL) instead. We'll be exploring the library and writing fuzzers in Rust in order to solve 
the challenges in a way that closely aligns with the suggested AFL++ usage.
