use itertools::Itertools;

/// Función para convertir índice lineal a par de nodos
pub fn index_to_edge(index: usize, n: usize) -> (usize, usize) {
    let mut sum = 0;
    let mut i = 0;

    // Encontrar la primera fila completa
    while sum + (n - i - 1) <= index {
        sum += n - i - 1;
        i += 1;
    }

    let j = index - sum + i + 1;
    (i, j)
}

/// Función para verificar si existe una arista entre dos nodos
pub fn is_connected(v: &[bool], n: usize, u: usize, w: usize) -> bool {
    if u == w {
        return false; // Un nodo no se conecta consigo mismo
    }

    let (min_node, max_node) = if u < w { (u, w) } else { (w, u) };

    // Calcular el índice en el vector v
    let offset = (2 * n - min_node - 1) * min_node / 2;
    let index = offset + (max_node - min_node - 1);

    if index < v.len() {
        v[index]
    } else {
        false
    }
}

/// Función para verificar si un conjunto de nodos forma un clique
pub fn is_clique(v: &[bool], n: usize, nodes: &[usize]) -> bool {
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            if !is_connected(v, n, nodes[i], nodes[j]) {
                return false;
            }
        }
    }
    true
}

/// Función principal para encontrar un clique de tamaño k en un grafo de n nodos
pub fn find_clique(v: &[bool], n: usize, k: usize) -> Option<Vec<usize>> {
    // Caso base: Si k == 1, cualquier nodo es un clique de tamaño 1
    if k == 1 {
        return Some(vec![0]);
    }

    // Caso base: Si k == n, todos los nodos deben formar un clique
    if k == n {
        if is_clique(v, n, &(0..n).collect::<Vec<_>>()) {
            return Some((0..n).collect());
        } else {
            return None;
        }
    }

    // Buscar todas las combinaciones posibles de k nodos
    (0..n).combinations(k).find(|nodes| is_clique(v, n, nodes))
}

/// Función wrapper que devuelve un booleano si existe un clique de tamaño k
pub fn clique(v: &[bool], n: usize, k: usize) -> bool {
    (k..n + 1).any(|i| find_clique(v, n, i).is_some())
}

/// Función para generar un grafo aleatorio con densidad especificada
pub fn generate_random_graph(n: usize, density: f64) -> Vec<bool> {
    use rand::random;
    let edges_count = n * (n - 1) / 2;
    let mut edges = Vec::with_capacity(edges_count);

    for _ in 0..edges_count {
        edges.push(random::<f64>() < density);
    }

    edges
}

/// Función para imprimir un grafo en formato de matriz de adyacencia
pub fn print_graph(v: &[bool], n: usize) {
    println!("Matriz de adyacencia:");
    for i in 0..n {
        for j in 0..n {
            if i == j {
                print!("0 ");
            } else {
                let connected = is_connected(v, n, i, j);
                print!("{} ", if connected { "1" } else { "0" });
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_to_edge() {
        // Para un grafo de 4 nodos:
        assert_eq!(index_to_edge(0, 4), (0, 1)); // Primer índice -> (0,1)
        assert_eq!(index_to_edge(1, 4), (0, 2)); // Segundo índice -> (0,2)
        assert_eq!(index_to_edge(2, 4), (0, 3)); // Tercer índice -> (0,3)
        assert_eq!(index_to_edge(3, 4), (1, 2)); // Cuarto índice -> (1,2)
        assert_eq!(index_to_edge(4, 4), (1, 3)); // Quinto índice -> (1,3)
        assert_eq!(index_to_edge(5, 4), (2, 3)); // Sexto índice -> (2,3)
    }

    #[test]
    fn test_is_connected() {
        let n = 4;
        let v = vec![true, false, true, true, false, true]; // Un grafo específico de 4 nodos

        assert!(is_connected(&v, n, 0, 1)); // v[0] es true
        assert!(!is_connected(&v, n, 0, 2)); // v[1] es false
        assert!(is_connected(&v, n, 0, 3)); // v[2] es true
        assert!(is_connected(&v, n, 1, 2)); // v[3] es true
        assert!(!is_connected(&v, n, 1, 3)); // v[4] es false
        assert!(is_connected(&v, n, 2, 3)); // v[5] es true
    }

    #[test]
    fn test_is_clique() {
        let n = 4;
        let v = vec![true, false, true, true, false, true]; // Un grafo específico de 4 nodos

        // {0, 1, 2} no es un clique porque 0 y 2 no están conectados
        assert!(!is_clique(&v, n, &[0, 1, 2]));

        // {0, 1, 3} no es un clique porque 1 y 3 no están conectados
        assert!(!is_clique(&v, n, &[0, 1, 3]));

        // {0, 3, 2} no es un clique porque 0 y 2 no están conectados
        assert!(!is_clique(&v, n, &[0, 3, 2]));

        // {1, 2, 3} no es un clique porque 1 y 3 no están conectados
        assert!(!is_clique(&v, n, &[1, 2, 3]));

        // {0, 1} es un clique de tamaño 2
        assert!(is_clique(&v, n, &[0, 1]));

        // {1, 2} es un clique de tamaño 2
        assert!(is_clique(&v, n, &[1, 2]));
    }
}
