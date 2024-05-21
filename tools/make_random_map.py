import argparse
import random
import json
from pathlib import Path

def make_note():
    lane = random.choice(['L1', 'L2', 'R1', 'R2'])
    return {
        'lane': lane
    }

def make_beat():
    note = make_note()
    return [note]

def make_beat_map(count: int):
    return [make_beat() for _ in range(count)]

def make_chart(beat_count: int, chart_name: str):
    sound_file = 'windless-slopes.ogg'
    beat_duration_secs = 0.3
    lead_time_secs = 1.5
    beats = make_beat_map(beat_count)
    return {
        'chart_name': chart_name,
        'sound_file': sound_file,
        'beat_duration_secs': beat_duration_secs,
        'lead_time_secs': lead_time_secs,
        'beats': beats
    }

def random_name(len=10):
    prefix = 'rand-map-'
    suffix = ''.join([
        random.choice('0123456789abcdef')
        for _ in range(len)
    ])
    return prefix + suffix

def main(beat_count: int, path: Path):
    chart_name = random_name()

    chart = make_chart(beat_count, chart_name)
    chart_json = json.dumps(chart)

    filename = f'{chart_name}.json'
    with open(path / filename, 'w') as file:
        file.write(chart_json)





    

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        prog='make_random_chart.py',
        description='Randomly generated charts for saffron-rhythm-duel game',
    )
    parser.add_argument('-n', '--number',
        dest='beat_count',
        required=True,
        type=int
    )
    args = parser.parse_args()

    path = Path(__file__).parent.parent / 'assets' / 'charts'

    main(args.beat_count, path)
