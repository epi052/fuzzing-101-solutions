# Fuzzing101 with LibAFL - Part Two

Herein lies a solution to [Exercise 2 from Fuzzing101](https://github.com/antonio-morales/Fuzzing101/tree/main/Exercise%202) written in Rust, using [LibAFL](https://github.com/AFLplusplus/LibAFL). 

The goal of the exercise is to find [CVE-2009-3895](https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2009-3895) and [CVE-2012-2836](https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2012-2836) in libexif 0.6.14.

The code housed here has a companion [blog post](https://epi052.gitlab.io/notes-to-self/blog/2021-11-07-fuzzing-101-with-libafl-part-2/) that delves into the different LibAFL components used in the solution.
