# Jargon

## Units

### Sky Coordinates

Sky position can be represented in ICRS[^1] (Right ascension [RA], Declination [Dec])

```py
from astropy import units as u
from astropy.coordinates import SkyCoord

c_icrs = SkyCoord(-1.1370067985279546, -1.1119959200661205, unit='rad')
# <SkyCoord (ICRS): (ra, dec) in deg
#    (294.85430917, -63.71267306)>

c_angle_deg = SkyCoord('19h39m25.027', '-63 42 45.61', unit=(u.hourangle, u.deg))
# <SkyCoord (ICRS): (ra, dec) in deg
#    (294.85427917, -63.71266944)>
```

[^1]https://en.wikipedia.org/wiki/International_Celestial_Reference_System_and_Frame

### Jansky

# Measurement Set

# Measurement Equation

[Measurement Equation](https://casadocs.readthedocs.io/en/stable/notebooks/casa-fundamentals.html#Measurement-Equation) and the need for calibration

# Imaging

## FITS

fits[^1] is the standard data format used in astronomy. A structure of a FITS file[^2] is as follows

- Primary header and data unit(HDU)
- Conforming Extensions (optional)
- Other records (optional)

An example header

```python
from astropy.utils.data import get_pkg_data_filename
from astropy.io import fits

image_file = get_pkg_data_filename('image.i.1934-638.beam00.taylor.0.restored.fits')

fits.info(image_file)
```

    Filename: image.i.1934-638.beam00.taylor.0.restored.fits
    No.    Name      Ver    Type      Cards   Dimensions   Format
      0  PRIMARY       1 PrimaryHDU      60   (2048, 2048, 1, 1)   float32   



[^1]https://fits.gsfc.nasa.gov/fits_documentation.html
[^2]https://fits.gsfc.nasa.gov/standard40/fits_standard40aa-le.pdf

## Direction

```
```
