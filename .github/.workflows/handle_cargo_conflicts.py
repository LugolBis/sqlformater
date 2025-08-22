import toml
import os

def handle_cargo_conflicts():
    # Charger le Cargo.toml de la branche python (actuel)
    with open('Cargo.toml', 'r') as f:
        python_toml = toml.load(f)
    
    # Charger le Cargo.toml de la branche main
    with open('.tmp_Cargo_main.toml', 'r') as f:
        main_toml = toml.load(f)
    
    # Préserver les configurations spécifiques à Python
    python_specific = {
        'lib': python_toml.get('lib', {}),
        'package': python_toml.get('package', {}),
        'dependencies': {
            'pyo3': python_toml.get('dependencies', {}).get('pyo3')
        }
    }
    
    # Fusionner les configurations
    merged_toml = main_toml.copy()
    
    # Restaurer les éléments spécifiques à Python
    if python_specific['lib']:
        merged_toml['lib'] = python_specific['lib']
    
    if python_specific['package']:
        merged_toml['package'] = {**merged_toml.get('package', {}), **python_specific['package']}
    
    if python_specific['dependencies']['pyo3']:
        if 'dependencies' not in merged_toml:
            merged_toml['dependencies'] = {}
        merged_toml['dependencies']['pyo3'] = python_specific['dependencies']['pyo3']
    
    # Écrire le fichier fusionné
    with open('Cargo.toml', 'w') as f:
        toml.dump(merged_toml, f)

if __name__ == '__main__':
    # Sauvegarder temporairement le Cargo.toml de main
    os.rename('Cargo.toml', '.tmp_Cargo_main.toml')
    
    # Récupérer le Cargo.toml de la branche python
    os.system('git checkout python_package -- Cargo.toml')  # Remplacez par votre branche
    
    # Fusionner les fichiers
    handle_cargo_conflicts()
    
    # Nettoyer
    os.remove('.tmp_Cargo_main.toml')