import libnum # install this with pip, not needed to run tests at all, only useful in adding TCs

bitsize=4096

r=libnum.randint_bits(bitsize)

print ("Random: %d Length: %d" % (r,libnum.len_in_bits(r)))


p=libnum.generate_prime(bitsize)

print ("\nPrime (p): %d. Length: %d bits, Digits: %d" % (p,libnum.len_in_bits(p), len(str(p)) ))  


q=libnum.generate_prime(bitsize)
print ("\nPrime (q): %d. Length: %d bits, Digits: %d" % (q,libnum.len_in_bits(q), len(str(q)) ))

N=p*q
print ("\nPrime (N): %d. Length: %d bits, Digits: %d" % (N,libnum.len_in_bits(N), len(str(N)) ))