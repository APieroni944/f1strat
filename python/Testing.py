import random
import math

def calculateOvertake(expected):
    return math.log(expected + 2.9, 8.6) + 0.005*expected + 0.175

expected = 10
overtake = calculateOvertake(expected)
#overtake = 1.86
actual = 0

#print(round(overtake, 3))

for lap in range(0, 7800):
    for driver in range(0, 21):
        gap = random.uniform(0.5, 5.0)
        deltapace = random.uniform(-0.4, 0.4)
        exponent = -8 * (deltapace - gap/overtake)
        probability = 1/(1 + math.e ** exponent)
        #print(probability)
        if random.random() < probability:
            actual += 0.01

print(f"Expected: {expected} \nActual:   {round(actual, 2)}")