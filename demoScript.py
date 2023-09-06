import cdg
import time
import pandas as pd

cdg.clear_cache()
start_time = time.time()
cdg.get_server_status()
a = time.time() - start_time
print(a)
df = pd.DataFrame(cdg.get_pub_treasury_data())
print(pd.DataFrame(df))
b = time.time() - start_time
print(b)
df = pd.DataFrame(cdg.get_pub_treasury_data())
print(pd.DataFrame(df))
c = time.time() - start_time
print(c)
print(f"First get without cached session took {b-a}.")
print(f"Second get took {c-(b+a)}, a diff of {b-c}!")
