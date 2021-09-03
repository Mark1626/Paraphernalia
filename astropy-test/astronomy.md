
## Terms

- **gridder** - Fourier Machines
  + Putting visible data into regular grids
  + Predicting visible data after applying FT
  + Computing PSF(Convolution functions)

- **solver**
  + Dirty - Inverts the data
  + Clean Solver - Cycle cleaning, done with algorithms

- **Point Spread Function**
  + It is the response of an imaging system to a point source
  + DeConvolution(check if it's correct) of the PSF and light source will produce

- **Pre Conditioning Function**
  + 

- **Visibility Point**

- **Non coplanar baseline**
  + Component of baseline in source direction
  + 

- **W-Projection**
  - W-projection is an algorithm used to solve Non Coplanar Baseline problem
  - [Wide Field Imaging Full Primary Beam](https://casa.nrao.edu/casadocs/casa-6.1.0/imaging/synthesis-imaging/wide-field-imaging-full-primary-beam)


- **Stacking Gridder**

## Imaging

Gridder + Solver = Imaging Algorithm

### Gridder

Different gridders use different convolution functions 

Gridders can be classified as `Mosaicing` and `Non-Mosaicing`.


### Solver

Algorithms
- `MultiScale`
- `MultiScaleMFS`

What the difference in CASA MultiScale


## References

### Slides

- [Fundamentals of Radio Astronomy](https://www.atnf.csiro.au/research/radio-school/2011/talks/Parkes-school-Fundamental-II.pdf)
- [Interferometry](https://www.eso.org/sci/meetings/2015/eris2015/L1_Jackson_Interferometry.pdf)
- [Radio Astronomy](http://aramis.obspm.fr/~salome/alma/lecture.pdf)

### Documentation Reference

- [Solver](https://www.atnf.csiro.au/computing/software/askapsoft/sdp/docs/current/calim/solver.html)
- [Gridder](https://www.atnf.csiro.au/computing/software/askapsoft/sdp/docs/current/calim/gridder.html)
- [Imaging Algorithms](https://casa.nrao.edu/casadocs/casa-6.1.0/imaging/synthesis-imaging/data-weighting)

### Example

- [Example](https://www.atnf.csiro.au/computing/software/askapsoft/sdp/docs/current/tutorials/basiccontinuum.html)

### Articles

- [PSF](https://telescope-optics.net/diffraction_image.htm)

### Wikipedia articles to understand terms

- [Convlution](https://en.wikipedia.org/wiki/Convolution)
- [Taylor Series](https://en.wikipedia.org/wiki/Taylor_series)
- [Rayleigh Jeans law](https://en.wikipedia.org/wiki/Rayleigh%E2%80%93Jeans_law)
- [Point Spread Function](https://en.wikipedia.org/wiki/Point_spread_function)
- [Prolate spheroidal wave function](https://en.wikipedia.org/wiki/Prolate_spheroidal_wave_function)
- [Window Function](https://en.wikipedia.org/wiki/Window_function)
- [Jansky](https://en.wikipedia.org/wiki/Jansky)

### Papers

- [W Projection](https://library.nrao.edu/public/memos/evla/legacy/evlamemo67.pdf)

