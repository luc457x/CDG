import cdg
import time
import pandas as pd

start_time = time.time()
while True:
    cdg.check_cached()
    cdg.get_sv_status()
    a = time.time() - start_time
    print(a)
    df = pd.DataFrame(cdg.get_pub_treasury_data())
    #print(pd.DataFrame(df))
    b = time.time() - start_time
    print(b)
    df = pd.DataFrame(cdg.get_pub_treasury_data())
    #print(pd.DataFrame(df))
    c = time.time() - start_time
    print(c)
    print(f"First get without cached session took {b-a}.")
    print(f"Second get took {c-(b+a)}, a diff of {b-c}!")
