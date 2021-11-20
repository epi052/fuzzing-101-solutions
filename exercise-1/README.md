# Fuzzing101 with LibAFL - Part One

Herein lies a solution to [Exercise 1 from Fuzzing101](https://github.com/antonio-morales/Fuzzing101/tree/main/Exercise%201) written in Rust, using [LibAFL](https://github.com/AFLplusplus/LibAFL). 

The goal of the exercise is to find a PoC for [CVE-2019-13288](https://www.cvedetails.com/cve/CVE-2019-13288/) in version 3.02 of [Xpdf](http://www.xpdfreader.com/index.html).

The code housed here has two companion blog posts that delve into the different LibAFL components used in the solution: 
- [Part I: Fuzzing Xpdf](https://epi052.gitlab.io/notes-to-self/blog/2021-11-01-fuzzing-101-with-libafl/)
- [Part I.V: Speed Improvements to Part I](https://epi052.gitlab.io/notes-to-self/blog/2021-11-07-fuzzing-101-with-libafl-part-1.5/)
