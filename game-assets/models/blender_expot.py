import bpy
import os
import re

# Base texture directory (update this)
TEXTURE_DIR = "/Users/jonnypower/Downloads/SourceFiles/Textures/"

# Get the current project file directory for exports
EXPORT_DIR = bpy.path.abspath("/Users/jonnypower/code/oncartforthee.ca/game-assets/models")

# Ensure export directory exists
if not os.path.exists(EXPORT_DIR):
    os.makedirs(EXPORT_DIR)

# Function to parse the .meta.tex file
def parse_meta_tex(file_path):
    texture_slots = []
    with open(file_path, "r") as file:
        for line in file:
            match = re.search(r"\(([^)]+)\)", line)  # Extract text inside parentheses
            if match:
                texture_name = match.group(1)
                texture_slots.append(texture_name)
    return texture_slots

# Function to apply textures to the object's material slots
def apply_textures(obj, texture_slots):
    if not obj.data.materials:
        print(f"Skipping {obj.name} (no materials found)")
        return
    
    materials = obj.data.materials
    
    for i, mat in enumerate(materials):
        if i >= len(texture_slots):  # Stop if there are more materials than textures
            break
        
        texture_name = texture_slots[i]
        texture_path = os.path.join(TEXTURE_DIR, texture_name + ".png")  # Assuming .png format
        
        if not os.path.exists(texture_path):
            print(f"Texture not found: {texture_path}")
            continue
        
        if mat.use_nodes:
            nodes = mat.node_tree.nodes
            image_node = None

            # Try to find an existing image texture node
            for node in nodes:
                if node.type == 'TEX_IMAGE':
                    image_node = node
                    break
            
            # If no image node exists, create one
            if not image_node:
                image_node = nodes.new(type="ShaderNodeTexImage")
                nodes["Principled BSDF"].inputs["Base Color"].default_value = (1, 1, 1, 1)  # Ensure a connection
                mat.node_tree.links.new(image_node.outputs["Color"], nodes["Principled BSDF"].inputs["Base Color"])
            
            # Assign the image
            try:
                image = bpy.data.images.load(texture_path)
                image_node.image = image
                print(f"Applied {texture_name} to {mat.name}")
            except:
                print(f"Failed to load texture: {texture_path}")

# Iterate over all objects in the scene
for obj in bpy.context.scene.objects:
    if obj.parent is None:  # Root objects only
        meta_file = os.path.join(EXPORT_DIR, f"{obj.name}.meta.tex")
        
        if not os.path.exists(meta_file):
            print(f"Skipping {obj.name} (meta file not found: {meta_file})")
            continue
        
        # Parse the .meta.tex file
        texture_slots = parse_meta_tex(meta_file)
        
        # Apply textures
        apply_textures(obj, texture_slots)

        # Select only the current object for export
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        bpy.context.view_layer.objects.active = obj
        
        # Define output file path
        file_path = os.path.join(EXPORT_DIR, f"{obj.name}.glb")
        
        # Export the object as a .glb file using current export settings
        bpy.ops.export_scene.gltf(
            filepath=file_path,
            export_format='GLB',
            use_selection=True  # Export only the selected object
        )

        print(f"Exported: {file_path}")

print("Export process completed.")