import subprocess, sys, re

# Run the grep command and capture output
def get_grep_output():
    result = subprocess.run(
        ["git", "grep", "-n", "-P", "^use"], capture_output=True, text=True
    )
    return result.stdout.strip().split("\n")

# Parse the grep output
def parse_grep_output(grep_lines):
    dependencies = []
    for line in grep_lines:
        if not line.strip():
            continue
        parts = line.split(":", 2)  # Split into filename, line number, and use statement
        if len(parts) == 3:
            source_file = parts[0]
            import_statement = parts[2].strip()
            dependencies.append((source_file, import_statement))
    return dependencies

def clean(dependencies):
    out = []
    for (source, use) in dependencies:
        use = use[4:]
        module = use.split('::')[0]
        if not (module in "crate super metalock_core"):
            continue
        source = source.replace("/src/", "/").replace(".rs", "").replace("-", "_").replace("/", "::")
        module = source.split('::', 2)[0]
        use = use.replace("crate::", module + "::")
        use = use.replace('super', "::".join(source.split("::")[:-1]))
        use = use.replace('::*;', '')
        use = re.sub('::{.*$', r"", use)
        use = re.sub('::[A-Z].*;', r"", use)
        use = re.sub('::impl_deref;', r"", use)
        out.append((source, use))
    return out
        

# Generate Graphviz dot content
def generate_graphviz(dependencies):
    graph_lines = ["digraph ImportDependencies {"]
    graph_lines.append("  rankdir=LR;")
    
    for source_label, import_label in clean(dependencies):
        # Simplify file name and import statement for clarity in graph
        #source_label = source_file.split("/")[-1]
        #import_label = import_statement.split(" ", 1)[-1]  # Remove 'use'
        graph_lines.append(f'  "{source_label}" -> "{import_label}";')

    graph_lines.append("}")
    return "\n".join(graph_lines)

# Main function
def main():
    grep_output = get_grep_output()
    dependencies = parse_grep_output(grep_output)
    graphviz_content = generate_graphviz(dependencies)

    # Save to a .dot file
    with open("import_dependencies.dot", "w") as dot_file:
        dot_file.write(graphviz_content)

    print("Graphviz file 'import_dependencies.dot' generated.")

if __name__ == "__main__":
    main()
