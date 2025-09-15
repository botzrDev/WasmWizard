use std::os::raw::c_char;
use std::ffi::CString;

// Recursive Fibonacci - O(2^n) time complexity
#[no_mangle]
pub extern "C" fn fib_recursive(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib_recursive(n - 1) + fib_recursive(n - 2),
    }
}

// Iterative Fibonacci - O(n) time, O(1) space complexity
#[no_mangle]
pub extern "C" fn fib_iterative(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut a = 0u64;
    let mut b = 1u64;

    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }

    b
}

// Matrix exponentiation - O(log n) time complexity
#[no_mangle]
pub extern "C" fn fib_matrix(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }

    // Use matrix exponentiation: [[1, 1], [1, 0]]^n
    let mut result = [[1u64, 0u64], [0u64, 1u64]]; // Identity matrix
    let mut base = [[1u64, 1u64], [1u64, 0u64]];   // Fibonacci matrix
    let mut exp = n;

    while exp > 0 {
        if exp % 2 == 1 {
            result = multiply_matrices(result, base);
        }
        base = multiply_matrices(base, base);
        exp /= 2;
    }

    result[0][1] // F(n) is at position [0][1]
}

// Helper function for matrix multiplication
fn multiply_matrices(a: [[u64; 2]; 2], b: [[u64; 2]; 2]) -> [[u64; 2]; 2] {
    let mut result = [[0u64; 2]; 2];

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                result[i][j] = result[i][j].wrapping_add(a[i][k].wrapping_mul(b[k][j]));
            }
        }
    }

    result
}

// Batch Fibonacci calculation - efficient for multiple values
#[no_mangle]
pub extern "C" fn fib_batch(start: u32, count: u32, result_ptr: *mut u64) {
    if result_ptr.is_null() || count == 0 {
        return;
    }

    // Calculate Fibonacci numbers from start to start+count-1
    for i in 0..count {
        let n = start + i;
        let fib = fib_iterative(n);
        unsafe {
            *result_ptr.add(i as usize) = fib;
        }
    }
}

// Performance comparison function
#[no_mangle]
pub extern "C" fn compare_algorithms(n: u32) -> *mut c_char {
    use std::time::Instant;

    // Time recursive algorithm (limit to avoid long execution)
    let recursive_time = if n <= 35 {
        let start = Instant::now();
        let _result = fib_recursive(n);
        start.elapsed().as_micros()
    } else {
        999999 // Indicate too slow
    };

    // Time iterative algorithm
    let start = Instant::now();
    let iterative_result = fib_iterative(n);
    let iterative_time = start.elapsed().as_micros();

    // Time matrix algorithm
    let start = Instant::now();
    let matrix_result = fib_matrix(n);
    let matrix_time = start.elapsed().as_micros();

    // Verify results match
    let results_match = iterative_result == matrix_result;

    let comparison = format!(
        "Fibonacci({}): Recursive={}μs, Iterative={}μs, Matrix={}μs, Results Match={}",
        n, recursive_time, iterative_time, matrix_time, results_match
    );

    let c_string = CString::new(comparison).unwrap();
    c_string.into_raw()
}

// Get algorithm information
#[no_mangle]
pub extern "C" fn get_algorithm_info(algorithm: u32) -> *mut c_char {
    let info = match algorithm {
        0 => "Recursive: O(2^n) time, O(n) space. Simple but slow for n > 30",
        1 => "Iterative: O(n) time, O(1) space. Fast and memory efficient",
        2 => "Matrix: O(log n) time, O(1) space. Fastest for large n",
        3 => "Batch: O(n) time for multiple values, efficient for sequences",
        _ => "Unknown algorithm",
    };

    let c_string = CString::new(info).unwrap();
    c_string.into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_sequence() {
        // Test first 10 Fibonacci numbers
        let expected = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34];

        for (n, &expected_val) in expected.iter().enumerate() {
            assert_eq!(fib_recursive(n as u32), expected_val as u64);
            assert_eq!(fib_iterative(n as u32), expected_val as u64);
            assert_eq!(fib_matrix(n as u32), expected_val as u64);
        }
    }

    #[test]
    fn test_large_fibonacci() {
        let n = 50;
        let result = fib_iterative(n);
        assert_eq!(result, 12586269025); // F(50)
    }

    #[test]
    fn test_algorithms_match() {
        for n in 0..30 {
            let recursive = fib_recursive(n);
            let iterative = fib_iterative(n);
            let matrix = fib_matrix(n);

            assert_eq!(recursive, iterative, "Algorithms don't match at n={}", n);
            assert_eq!(iterative, matrix, "Algorithms don't match at n={}", n);
        }
    }

    #[test]
    fn test_batch_calculation() {
        let mut results = [0u64; 10];
        let results_ptr = results.as_mut_ptr();

        fib_batch(0, 10, results_ptr);

        let expected = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
        for (i, &expected_val) in expected.iter().enumerate() {
            assert_eq!(results[i], expected_val as u64);
        }
    }
}