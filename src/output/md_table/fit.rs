use unicode_width::UnicodeWidthStr;

pub(crate) fn column_widths(rows: &[Vec<String>], max_width: usize) -> Vec<usize> {
    let natural = natural_widths(rows);
    let minimums = minimum_widths(rows);
    constrain(&natural, &minimums, max_width)
}

fn natural_widths(rows: &[Vec<String>]) -> Vec<usize> {
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0; cols];
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(cell.width());
        }
    }
    widths
}

fn minimum_widths(rows: &[Vec<String>]) -> Vec<usize> {
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0; cols];
    for row in rows {
        for (idx, cell) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(longest_word(cell));
        }
    }
    widths
}

fn longest_word(cell: &str) -> usize {
    cell.split_whitespace()
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0)
}

fn constrain(natural: &[usize], minimums: &[usize], max_width: usize) -> Vec<usize> {
    if max_width == 0 || natural.is_empty() {
        return natural.to_vec();
    }
    let available = max_width.saturating_sub(chrome_width(natural.len()));
    if natural.iter().sum::<usize>() <= available {
        return natural.to_vec();
    }
    let floors = floors_of(natural, minimums);
    if floors.iter().sum::<usize>() <= available {
        return fit_above_floors(natural, &floors, available);
    }
    fit_below_floors(natural, &floors, available)
}

fn floors_of(natural: &[usize], minimums: &[usize]) -> Vec<usize> {
    natural
        .iter()
        .zip(minimums)
        .map(|(width, min)| (*min).min(*width))
        .collect()
}

fn fit_above_floors(natural: &[usize], floors: &[usize], available: usize) -> Vec<usize> {
    let mut scaled = floors.to_vec();
    let extra = available.saturating_sub(scaled.iter().sum());
    let slack: Vec<usize> = natural
        .iter()
        .zip(&scaled)
        .map(|(width, used)| width.saturating_sub(*used))
        .collect();
    distribute(&mut scaled, &slack, extra, natural);
    scaled
}

fn fit_below_floors(natural: &[usize], floors: &[usize], available: usize) -> Vec<usize> {
    let mut scaled: Vec<usize> = natural
        .iter()
        .map(|width| usize::from(*width > 0))
        .collect();
    let Some(mut budget) = available.checked_sub(scaled.iter().sum()) else {
        return scaled;
    };
    if budget == 0 {
        return scaled;
    }
    budget = give_short_columns(&mut scaled, natural, floors, budget);
    grow_widest(&mut scaled, natural, budget);
    scaled
}

fn give_short_columns(
    scaled: &mut [usize],
    natural: &[usize],
    floors: &[usize],
    mut budget: usize,
) -> usize {
    let long: Vec<usize> = floors
        .iter()
        .enumerate()
        .filter(|(_, floor)| **floor > 12)
        .map(|(idx, _)| idx)
        .collect();
    if long.is_empty() {
        return budget;
    }
    let mut short: Vec<usize> = floors
        .iter()
        .enumerate()
        .filter(|(idx, floor)| natural[*idx] > 0 && **floor <= 12)
        .map(|(idx, _)| idx)
        .collect();
    short.sort_by_key(|&idx| floors[idx]);
    for idx in short {
        budget = give_one_short(GiveShort {
            scaled,
            floors,
            long: &long,
            idx,
            budget,
        });
    }
    budget
}

struct GiveShort<'a> {
    scaled: &'a mut [usize],
    floors: &'a [usize],
    long: &'a [usize],
    idx: usize,
    budget: usize,
}

fn give_one_short(args: GiveShort<'_>) -> usize {
    let need = args.floors[args.idx].saturating_sub(args.scaled[args.idx]);
    if need == 0 {
        return args.budget;
    }
    let reserve = args
        .long
        .iter()
        .filter(|idx| args.scaled[**idx] < args.floors[**idx])
        .count();
    let give = need.min(args.budget.saturating_sub(reserve));
    if give == 0 {
        return args.budget;
    }
    args.scaled[args.idx] += give;
    args.budget - give
}

fn grow_widest(scaled: &mut [usize], natural: &[usize], mut budget: usize) {
    while budget > 0 {
        let Some(idx) = widest_slack(natural, scaled) else {
            break;
        };
        if natural[idx] <= scaled[idx] {
            break;
        }
        scaled[idx] += 1;
        budget -= 1;
    }
}

fn distribute(scaled: &mut [usize], slack: &[usize], extra: usize, natural: &[usize]) {
    let slack_total: usize = slack.iter().sum();
    if extra == 0 || slack_total == 0 {
        return;
    }
    let mut given = 0usize;
    for (idx, amount) in slack.iter().enumerate() {
        let share = amount.saturating_mul(extra) / slack_total;
        scaled[idx] += share;
        given += share;
    }
    grow_widest(scaled, natural, extra.saturating_sub(given));
}

fn widest_slack(natural: &[usize], scaled: &[usize]) -> Option<usize> {
    natural
        .iter()
        .zip(scaled)
        .enumerate()
        .max_by_key(|(_, (width, used))| width.saturating_sub(**used))
        .map(|(idx, _)| idx)
}

const fn chrome_width(cols: usize) -> usize {
    cols.saturating_mul(3).saturating_add(1)
}
