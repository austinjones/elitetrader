# Elite Trader
This program collects information about your status in the game Elite Dangerous,
and calculates the best possible trade route.

It is useful to set the **-c** Credit, **-r** Jump Range, **-p** Ship Size, and 
**-m** Min Balance args on the command line, as they don't change often.
The tool will interactively prompt you for any remaining information,
such as the Current Station and Credit Balance.

Set Minimum Balance argument carefully.  You should allow your rebuy
cost plus a full load of expensive cargo, or two of each to be safe.
If you set the balance too low, you might end up broke in a Sidewinder!

## Usage
```
    -t --station GitHub current station name
    -c --cargo 216      maximum cargo capacity in tons. find this in your
                        right cockpit panel's Cargo tab.
    -r --range 18.52    maximum laden jump range in light years. find this in
                        your outfitting menu.
    -b --balance 525.4k current credit balance
    -m --minbalance 3.5m
                        minimum credit balance - safety net for rebuy
    -q --quality med    search quality setting [med|high|ultra]
    -p --shipsize large current ship size (small|med|large)
    -d --debug 12       searches to the given hop length and prints stats
    -i --timetables     prints time tables
    -h --help           prints this help menu
```
## Algorithm
The top few trades are calculated from your current station, and the
top few trades from those stations are calculated, and so on.
From all the possible trade routes, the first trade from 
the best route is presented to you.

The Search Quality flag -q affects the depth of the search process.

Possible trades are scored by their total profit per minute,
which the program estimates based on your ship's jump range.

Here is an example trade hop.  In this example, we start at Giger Hub,
and buy Palladium.  We take it to Iben Dock in Peraesii.
Once the trade is complete, we press <enter> and the credit/min
analysis is printed.

```
-------------------------------------------------------------------
Starting route search from Peraesii [Giger Hub] ...
hop 1:	Peraesii [Giger Hub]

buy:	216x Palladium [Metals]
sell:	Chono [Siddha Ring]
248.9 Kcr profit for balance 15.25 Mcr

expect:	28.0 Kcr profit/min over 9.1 mins
1.2 Kcr profit/ton for 216 tons
25.8 ly to system, 9 ls to station

wait:	press <enter> once trade is complete.

actual:	29.1 Kcr per min over 8.6 minutes
5.82% faster than expected
-------------------------------------------------------------------
```

We made 29 thousand credits per minute over 8.6 minutes.
Give it a try with your ship!
