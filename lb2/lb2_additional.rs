use std::{f64, io};

fn main() {
    loop {
        println!("Введіть вираз в польській нотації (наприклад, '+ 1 2' або '* + 1 2 + 3 4') або введіть 'off' для виходу:");

        let expression = get_input_str().trim().to_lowercase();
        if expression == "off" {
            println!("Вихід...");
            break;
        }

        let mut tokens = expression.split_whitespace().collect::<Vec<&str>>();
        match evaluate_polish_notation(&mut tokens) {
            Ok(result) => println!("Результат: {}", result),
            Err(e) => println!("Помилка: {}", e),
        }
    }
}

fn evaluate_polish_notation(tokens: &mut Vec<&str>) -> Result<f64, String> {
    if tokens.is_empty() {
        return Err("Недостатньо операндів".to_string());
    }

    let token = tokens.remove(0); 
    
    match token {
        "+" | "-" | "*" | "/" => {
            let left_operand = evaluate_polish_notation(tokens)?;
            let right_operand = evaluate_polish_notation(tokens)?;

            match token {
                "+" => Ok(left_operand + right_operand),
                "-" => Ok(left_operand - right_operand),
                "*" => Ok(left_operand * right_operand),
                "/" => {
                    if right_operand == 0.0 {
                        Err("Помилка: ділення на нуль".to_string())
                    } else {
                        Ok(left_operand / right_operand)
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => match token.parse::<f64>() {
            Ok(num) => Ok(num),
            Err(_) => Err("Недійсний операнд або оператор".to_string()),
        },
    }
}

fn get_input_str() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Помилка при зчитанні даних");
    input
}
