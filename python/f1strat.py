import math
import os
import fastf1
from datetime import date
import pandas as pd
import matplotlib.pyplot as plt

class Driver:
    def __init__(self, Number, Lap, Tire, Fuel, Totaltime, Optlap, isStuck, Strategy):
        self.number = Number
        self.lap = Lap
        self.tire = Tire
        self.fuel = Fuel
        self.totaltime = Totaltime
        self.optlap = Optlap
        self.isstuck = isStuck
        self.strat = Strategy
    #def writeMsgpack(self)
    def printData(self, position):
        print(f"{position}.  Number {self.number}  Optlap {self.optlap}")

class Track:
    def __init__(self, Laps, expected, Pitloss, Wearmod):
        self.laps = Laps
        self.overtake = self.calculateOvertake(expected)
        self.pitloss = Pitloss
        self.wearmod = Wearmod
    def calculateOvertake(self, expected):
        return math.log(expected + 2.9, 8.6) + 0.005 * expected + 0.175


class Strategy:
    def __init__(self, Tire, Lap, StdDev):
        self.tire = Tire
        self.lap = Lap
        self.sd = StdDev


def getData(race):
    year = date.today().year
    session = fastf1.get_session(year, race, 'Q')
    session.load(laps = True, telemetry=False, weather=False, messages=False)
    drivers = []
    tire = (1, 0.0)
    strat = []
    strat.append(Strategy(2, 30, 2))
    #print(session.laps.to_string())

    for index, row in session.results.iterrows():
        number = row['DriverNumber']
        optlap = getBestLap(row)
        driver = Driver(number, 0.0, tire, 1.0, 0.0, optlap, True, strat)
        drivers.append(driver)
    return drivers

def getBestLap(row):
    if pd.notna(row['Q3']):
        return row['Q3'].total_seconds()
    elif pd.notna(row['Q2']):
        return row['Q2'].total_seconds()
    elif pd.notna(row['Q1']):
        return row['Q1'].total_seconds()
    else:
        return 999999

def getTrack(race):
    year = date.today().year - 1
    session = fastf1.get_session(year, race, 'R')
    session.load(laps = True, telemetry=False, weather=False, messages=False)
    laps = session.total_laps
    expected = -1
    while expected == -1:
        try:
            expected = int(input("Enter the expected number of on track overtakes for the race: "))
        except Exception as e:
            print(f"ERRROR: {e} \nPlease try again")
    pitloss = -1
    while pitloss == -1.0:
        try:
            pitloss = float(input("Enter the expected number of on track overtakes for the race: "))
        except Exception as e:
            print(f"ERRROR: {e} \nPlease try again")
    wearmod = 1
    try:
        expected = float(input("Enter the tire wear rate (Default 1): "))
    except Exception as e:
        print(f"ERRROR: {e} \nContinuing...")
    return Track(laps, expected, pitloss, wearmod)

def plotData(drivers):
    y = []
    for driver in drivers:
        y.append(driver.optlap)
    x = range(0, len(drivers))
    plt.plot(x, y)
    plt.savefig("plot.png")

def main():
    race = input("Input race (eg.'Monaco'): ")
    drivers = getData(race)
    #for i in range(0, len(drivers)):
    #    drivers[i].printData(i + 1)
    plotData(drivers)
    track = getTrack(race)

if __name__ == '__main__':
    main()
