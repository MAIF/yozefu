where ((topic == "system" and key contains "restart") 
   or !(value starts with "ignored" and partition > 2)) 
  and (timestamp between "3 hours ago" and "20 minutes ago")
  and myFilter("check", "error", 500) or myFilter("type", 100)
order by key desc
limit 50
from beginning