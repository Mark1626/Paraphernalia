# Decoding Tom's Onion Data

https://www.tomdalling.com/toms-data-onion/

## Layer 0 - Ascii 85

Straight forward

## Layer 1 - XOR

Needed a custom mask and bit operation for rotate right

## Layer 2 - Parity

Wrote a push decoder for decoding the bytes. Wrote a state machine initially, but it was unnecessary

## Layer 3

Used an english letter and work ranking, a header analysis for finding the key

## Layer 4 - UDP

Pretty fun layer to work hands on, found it simpler than the previous layer

## Layer 5 - Advanced Encryption Standard

AES Wrap algorithm implementation needed

## Layer 6 - Virtual Machine - TODO
