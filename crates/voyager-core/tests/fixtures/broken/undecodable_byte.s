; EXPECT: InvalidEncoding
PRINT LIST='before'
; a comment containing an undecodable byte -> Å <- right there
PRINT LIST='after'
