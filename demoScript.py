import cdg
import time

start_time = time.time()
print(cdg.get_server_status())
print("--- %s seconds ---" % (time.time() - start_time))