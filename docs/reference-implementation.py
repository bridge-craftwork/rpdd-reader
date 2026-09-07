import sys
M32 = 0xFFFFFFFF
TOTAL = 0xad55e315634dda658bf49200   # 52!/(13!)^4

class G:
    __slots__=('s0','s1','s2','s3','c')
    def __init__(self): self.s0=self.s1=self.s2=self.s3=self.c=0

    def rand32(self):
        p = (0x13FB*self.s0 + 0x6F0*self.s1 + 0x5D4*self.s2
             + 0x7DD4FFC7*self.s3 + self.c)
        self.s3=self.s2; self.s2=self.s1; self.s1=self.s0
        self.s0 = p & M32
        self.c  = p >> 32
        return self.s0

    def seed(self, n):
        # 0x4011BF, entered with EAX = deal counter; n = (counter>>14)+1
        edx = n & M32
        eax = (n ^ 0xFFFF) & M32
        eax = ((eax >> 10) | (eax << 22)) & M32          # ror eax,10
        eax = (eax & 0xFFFF0000) | (edx & 0xFFFF)        # mov ah,dh / mov al,dl
        eax = ((eax >> 10) | (eax << 22)) & M32          # ror eax,10
        edx = (n << 2) & M32                             # shl edx,2
        eax |= (edx & 0xFF00)                            # or ah,dh
        eax |= (edx & 0x00FF)                            # or al,dl
        eax |= 3                                         # or al,3
        w = []
        for _ in range(5):
            eax = (eax * 0x1C8E815) & M32                # imul esi
            eax = (eax - 1) & M32                        # dec eax
            w.append(eax)                                # stosd
        self.s0,self.s1,self.s2,self.s3,self.c = w
        for _ in range(12):                              # 12 warm-up calls
            self.rand32()

def deal(g, counter):
    if (counter & 0x3FFF) == 0:
        g.seed((counter >> 14) + 1)
    r0 = g.rand32(); r1 = g.rand32(); r2 = g.rand32()
    m = 0xAD55E316
    if (r1 << 32 | r0) >= 0x634DDA658BF49200:
        m -= 1
    R = ((r2 * m) >> 32) << 64 | (r1 << 32) | r0
    P = TOTAL
    cnt = [13,13,13,13]        # index i -> 0x402014+i ; seat = 3-i
    cards = [0]*52             # card j (0=SA) <-> ebp = 52-j
    for ebp in range(52, 0, -1):
        i = 3
        Q = (P * cnt[i]) // ebp
        while Q <= R and i > 0:
            R -= Q
            i -= 1
            Q = (P * cnt[i]) // ebp
        cnt[i] -= 1
        cards[52-ebp] = 3 - i
        P = Q
    b = bytearray(13)
    for j in range(52):
        b[j >> 2] |= cards[j] << (2*(j & 3))
    return bytes(b)

if __name__ == '__main__':
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 512
    start = int(sys.argv[2]) if len(sys.argv) > 2 else 0
    g = G()
    out = bytearray()
    # to reach `start` honestly we must run from 0 unless start is 16K-aligned
    for i in range(start):
        deal(g, i)
    for i in range(start, start+n):
        out += deal(g, i)
    sys.stdout.buffer.write(bytes(out))
