def greet(name: str) -> str:
    """Return a greeting for a trimmed name, rejecting blank names."""
    name = name.strip()
    if not name:
        raise ValueError("name must not be empty or whitespace")
    return f"Hello, {name}!"
