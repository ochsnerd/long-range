# necessary code to generate the measurement based on the simulation
import random
import math
from typing import Iterator
import concurrent.futures

from union_find import UnionFind

# FIXME: is the geometric skip sampled correctly? 
def geometric_skip(p:float, rng: random.Random) -> int:
    """samples the geometric skip in terms of p for skip sampling"""
    if p >= 1.0:
        return 0
    if p <= 1e-15:
        return 10**20  # very large number (just needs to skip everything) # FIXME; is there something more elegant?
    u = rng.random()
    return int(math.floor(math.log(u) / math.log(1 - p)))


# compute the largestset and return the UF structure (compute the measurements afterwards for greater flexibility)
def lr_perco_config(N: int,  #FIXME should actually be the L (less confusion)
                    alpha: float,
                    beta: float,
                    seed = None,
                    d: int = 2
    ) -> UnionFind:
    rng = random.Random(seed)
    tot_pts = N ** d
    uf = UnionFind(tot_pts)

    # write functions to transform coordinates to linear order index of cluster points
    def coord_to_idx(coords: list[int]) -> int:
        idx = 0
        for i, x in enumerate(coords):
            idx += x * (N ** (d-1-i))
        return idx
    
    def idx_to_coord(idx: int) -> list[int]:
        coords = [0] * d
        temp = idx
        for i in range(d-1, -1, -1):
            coords[i] = temp % N
            temp //= N
        return coords
    
    # create a generator which goes through all the necessary directions
    #def directions() -> Iterator[list[int]]:
    #    direction = [0] * d
    #    while True:
    #        for i in range(d-1, -1, -1):
    #            direction[i] += 1
    #            if direction[i] < N:
    #                break
    #            direction[i] = 0

    #        if all(x == 0 for x in direction):
    #            break
    #        yield direction
    def directions() -> Iterator[list[int]]:
        half = N // 2
        # iterative odometer over [0..half]^d, skipping all-zero
        r = [0]*d
        first = True
        while True:
            if not first and any(r):
                yield r.copy()
            first = False
            # increment r
            for i in range(d-1, -1, -1):
                r[i] += 1
                if r[i] <= half:
                    break
                r[i] = 0
            else:
                return
    
    def d_T(displ: list[int]) -> float:
        distance = 0
        for displ_j in displ:
            distance += abs(min(displ_j, N - displ_j))
        return distance

    half = N // 2
    
    for jump_dir in directions():
        jump_dist = d_T(jump_dir)

        if jump_dist < 1e-12:
            continue
        
        p = min(1.0, beta / (jump_dist ** (d + alpha)))
        if p <= 0.0:
            continue
        # ---- self-complement handling (r == -r mod N) ----
        self_comp = (N % 2 == 0) and any(x == half for x in jump_dir) and all(x in (0, half) for x in jump_dir)
        mid_axis = -1
        if self_comp:
            # choose a canonical axis where r_k = N/2
            for k, x in enumerate(jump_dir):
                if x == half:
                    mid_axis = k
                    break
        i = 0
        while i < tot_pts:
            i += geometric_skip(p, rng=rng)
            if i >= tot_pts:
                break

            if self_comp:
                coords = idx_to_coord(i)
                if coords[mid_axis] >= half:  # keep only half the bases
                    i += 1
                    continue
            else:
                coords = idx_to_coord(i)

            j = coord_to_idx([(coords[l] + jump_dir[l]) % N for l in range(d)])
            uf.union(i, j)
            i += 1

    return uf

def measurement(uf: UnionFind, L: int, d: int = 2) -> tuple[float, float]: # returns (QG, S)
    """
    compute the spread of the cluster size (Q_G) - eq (6) and S form the paper
    https://arxiv.org/pdf/1610.00200
    """
    unique_roots = set()
    for i in range(L ** d):
        unique_roots.add(uf.find(i))
    # square sum
    sq_sum = 0
    cube_sum = 0 
    for r in unique_roots:  # ideally compute all measurements here
        sq_sum += uf.size[r]**2
        cube_sum += uf.size[r]**4
    return cube_sum / (sq_sum ** 2), sq_sum / (L** d)

# parallelized implementation of the computation of one dataframe/measurement
def _measurement(i: int, L: int, alpha: float, beta: float, d: int
                 ) -> tuple[float, float]:
    """helper function for a single measurement of Q_G and S. Function 
    is used to map parameters of the simulation over the parameter array"""
    uf = lr_perco_config(N=L, alpha=alpha, beta=beta, d=d)
    q, s = measurement(uf, L=L,d=d)
    return q, s

# measure over the product of: L x beta x d x N (?)@ fixed alpha, d
# only needs to be an iterable
def m_run(N, L, alpha, beta, d):
    with concurrent.futures.ProcessPoolExecutor() as ex:
       it = ex.map(_measurement, range(N), 
                   [L]*N, [alpha]*N, [beta]*N, [d]*N)
       qs, ss = [], []
       for q, s in it:
           qs.append(q)
           ss.append(s) 
    return  qs, ss