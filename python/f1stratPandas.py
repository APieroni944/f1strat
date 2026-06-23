import pandas as pd
import fastf1
from datetime import date

#def simulate(drivers, track):
    #simulate race using rust
def getData(race):
    year = date.today().year
    session = fastf1.get_session(year, race, 'Q')
    session.load()

    driver = session.results[['DriverNumber', 'BestLapTime']].copy()
    driver.columns = ['number', 'optlap']


def main():
    fastf1.Cache.enable_cache
    try:
        race = input("Enter the race you want to simulate: ")
        StartData = getData()
    except Exception as e:
        print("Error getting session: ", e)
    print(session.results)

    
if __name__ == '__main__':
    main()
